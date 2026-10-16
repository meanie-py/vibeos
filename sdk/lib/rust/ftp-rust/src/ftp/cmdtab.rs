use crate::ftp::ftp::data;
use crate::ftp::domacro::domacro;
use crate::ftp::fake::glob;
use crate::ftp::ftp::null;
use crate::ftp::ftp::reset;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut accounthelp: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"send account command to remote server\0",
    )
};

#[no_mangle]
pub static mut appendhelp: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"append to a file\0")
};

#[no_mangle]
pub static mut asciihelp: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"set ascii transfer type\0")
};

#[no_mangle]
pub static mut beephelp: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"beep when command completed\0")
};

#[no_mangle]
pub static mut binaryhelp: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"set binary transfer type\0")
};

#[no_mangle]
pub static mut casehelp: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"toggle mget upper/lower case id mapping\0",
    )
};

#[no_mangle]
pub static mut cdhelp: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"change remote working directory\0",
    )
};

#[no_mangle]
pub static mut cduphelp: [::core::ffi::c_char; 52] = unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
        *b"change remote working directory to parent directory\0",
    )
};

#[no_mangle]
pub static mut chmodhelp: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"change file permissions of remote file\0",
    )
};

#[no_mangle]
pub static mut connecthelp: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"connect to remote tftp\0")
};

#[no_mangle]
pub static mut crhelp: [::core::ffi::c_char; 47] = unsafe {
    ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
        *b"toggle carriage return stripping on ascii gets\0",
    )
};

#[no_mangle]
pub static mut deletehelp: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"delete remote file\0")
};

#[no_mangle]
pub static mut debughelp: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"toggle/set debugging mode\0")
};

#[no_mangle]
pub static mut dirhelp: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"list contents of remote directory\0",
    )
};

#[no_mangle]
pub static mut disconhelp: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"terminate ftp session\0")
};

#[no_mangle]
pub static mut domachelp: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"execute macro\0") };

#[no_mangle]
pub static mut formhelp: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"set file transfer format\0")
};

#[no_mangle]
pub static mut globhelp: [::core::ffi::c_char; 51] = unsafe {
    ::core::mem::transmute::<[u8; 51], [::core::ffi::c_char; 51]>(
        *b"toggle metacharacter expansion of local file names\0",
    )
};

#[no_mangle]
pub static mut hashhelp: [::core::ffi::c_char; 48] = unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
        *b"toggle printing `#' for each buffer transferred\0",
    )
};

#[no_mangle]
pub static mut helphelp: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"print local help information\0",
    )
};

#[no_mangle]
pub static mut idlehelp: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"get (set) idle timer on remote side\0",
    )
};

#[no_mangle]
pub static mut lcdhelp: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"change local working directory\0",
    )
};

#[no_mangle]
pub static mut lshelp: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"list contents of remote directory\0",
    )
};

#[no_mangle]
pub static mut macdefhelp: [::core::ffi::c_char; 15] =
    unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"define a macro\0") };

#[no_mangle]
pub static mut mdeletehelp: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"delete multiple files\0")
};

#[no_mangle]
pub static mut mdirhelp: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"list contents of multiple remote directories\0",
    )
};

#[no_mangle]
pub static mut mgethelp: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"get multiple files\0")
};

#[no_mangle]
pub static mut mkdirhelp: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"make directory on the remote machine\0",
    )
};

#[no_mangle]
pub static mut mlshelp: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"list contents of multiple remote directories\0",
    )
};

#[no_mangle]
pub static mut modtimehelp: [::core::ffi::c_char; 43] = unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"show last modification time of remote file\0",
    )
};

#[no_mangle]
pub static mut modehelp: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"set file transfer mode\0")
};

#[no_mangle]
pub static mut mputhelp: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"send multiple files\0")
};

#[no_mangle]
pub static mut newerhelp: [::core::ffi::c_char; 50] = unsafe {
    ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
        *b"get file if remote file is newer than local file \0",
    )
};

#[no_mangle]
pub static mut nlisthelp: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"nlist contents of remote directory\0",
    )
};

#[no_mangle]
pub static mut nmaphelp: [::core::ffi::c_char; 44] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"set templates for default file name mapping\0",
    )
};

#[no_mangle]
pub static mut ntranshelp: [::core::ffi::c_char; 52] = unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
        *b"set translation table for default file name mapping\0",
    )
};

#[no_mangle]
pub static mut porthelp: [::core::ffi::c_char; 48] = unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
        *b"toggle use of PORT cmd for each data connection\0",
    )
};

#[no_mangle]
pub static mut prompthelp: [::core::ffi::c_char; 49] = unsafe {
    ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
        *b"force interactive prompting on multiple commands\0",
    )
};

#[no_mangle]
pub static mut proxyhelp: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"issue command on alternate connection\0",
    )
};

#[no_mangle]
pub static mut pwdhelp: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"print working directory on remote machine\0",
    )
};

#[no_mangle]
pub static mut quithelp: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"terminate ftp session and exit\0",
    )
};

#[no_mangle]
pub static mut quotehelp: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"send arbitrary ftp command\0")
};

#[no_mangle]
pub static mut receivehelp: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"receive file\0") };

#[no_mangle]
pub static mut regethelp: [::core::ffi::c_char; 41] = unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"get file restarting at end of local file\0",
    )
};

#[no_mangle]
pub static mut remotehelp: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"get help from remote server\0")
};

#[no_mangle]
pub static mut renamehelp: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"rename file\0") };

#[no_mangle]
pub static mut restarthelp: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"restart file transfer at bytecount\0",
    )
};

#[no_mangle]
pub static mut rmdirhelp: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"remove directory on the remote machine\0",
    )
};

#[no_mangle]
pub static mut rmtstatushelp: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"show status of remote machine\0",
    )
};

#[no_mangle]
pub static mut runiquehelp: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"toggle store unique for local files\0",
    )
};

#[no_mangle]
pub static mut resethelp: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"clear queued command replies\0",
    )
};

#[no_mangle]
pub static mut sendhelp: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"send one file\0") };

#[no_mangle]
pub static mut passivehelp: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"enter passive transfer mode\0")
};

#[no_mangle]
pub static mut sitehelp: [::core::ffi::c_char; 99] = unsafe {
    ::core::mem::transmute::<
        [u8; 99],
        [::core::ffi::c_char; 99],
    >(
        *b"send site specific command to remote server\n\t\tTry \"rhelp site\" or \"site help\" for more information\0",
    )
};

#[no_mangle]
pub static mut shellhelp: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"escape to the shell\0")
};

#[no_mangle]
pub static mut sizecmdhelp: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"show size of remote file\0")
};

#[no_mangle]
pub static mut statushelp: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"show current status\0")
};

#[no_mangle]
pub static mut structhelp: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"set file transfer structure\0")
};

#[no_mangle]
pub static mut suniquehelp: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"toggle store unique on remote machine\0",
    )
};

#[no_mangle]
pub static mut systemhelp: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"show remote system type\0")
};

#[no_mangle]
pub static mut tenexhelp: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"set tenex file transfer type\0",
    )
};

#[no_mangle]
pub static mut tracehelp: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"toggle packet tracing\0")
};

#[no_mangle]
pub static mut typehelp: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"set file transfer type\0")
};

#[no_mangle]
pub static mut umaskhelp: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"get (set) umask on remote side\0",
    )
};

#[no_mangle]
pub static mut userhelp: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"send new user information\0")
};

#[no_mangle]
pub static mut verbosehelp: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"toggle verbose mode\0")
};

#[no_mangle]
pub static mut cmdtab: [cmd; 73] = unsafe {
    [
        cmd {
            c_name: b"!\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const shellhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                shell
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"$\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const domachelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                domacro
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"account\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const accounthelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                account
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"append\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const appendhelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                put as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"ascii\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const asciihelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setascii
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"bell\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const beephelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                setbell
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"binary\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const binaryhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setbinary
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"bye\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const quithelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                quit as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"case\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const casehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setcase
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"cd\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const cdhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                cd as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"cdup\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const cduphelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                cdup as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"chmod\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const chmodhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                do_chmod
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"close\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const disconhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                disconnect
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"cr\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const crhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                setcr
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"delete\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const deletehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                delete
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"debug\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const debughelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                setdebug
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"dir\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const dirhelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                ls as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"disconnect\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const disconhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                disconnect
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"form\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const formhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setform
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"get\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const receivehelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                get,
            ),
        },
        cmd {
            c_name: b"glob\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const globhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                setglob
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"hash\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const hashhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                sethash
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"help\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const helphelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                help as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"idle\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const idlehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                idle as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"image\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const binaryhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setbinary
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"lcd\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const lcdhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                lcd as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"ls\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const lshelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                ls as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"macdef\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const macdefhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                macdef
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"mdelete\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const mdeletehelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                mdelete
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"mdir\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const mdirhelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                mls as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"mget\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const mgethelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                mget as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"mkdir\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const mkdirhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                makedir
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"mls\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const mlshelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                mls as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"mode\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const modehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                fsetmode
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"modtime\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const modtimehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                modtime
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"mput\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const mputhelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                mput as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"newer\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const newerhelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                newer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"nmap\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const nmaphelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setnmap
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"nlist\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const nlisthelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                ls as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"ntrans\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const ntranshelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setntrans
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"open\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const connecthelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setpeer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"passive\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const passivehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                setpassive
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"prompt\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const prompthelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                setprompt
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"proxy\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const proxyhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                doproxy
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"sendport\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const porthelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                setport
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"put\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const sendhelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                put as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"pwd\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const pwdhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                pwd as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"quit\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const quithelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                quit as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"quote\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const quotehelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                quote
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"recv\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const receivehelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                get,
            ),
        },
        cmd {
            c_name: b"reget\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const regethelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                reget
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"rstatus\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const rmtstatushelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                rmtstatus
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"rhelp\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const remotehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                rmthelp
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"rename\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const renamehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                renamefile
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"reset\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const resethelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                reset
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"restart\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const restarthelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                restart
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"rmdir\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const rmdirhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                removedir
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"runique\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const runiquehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setrunique
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"send\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const sendhelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                put as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"site\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const sitehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                site as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"size\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const sizecmdhelp as *mut ::core::ffi::c_char,
            c_bell: 1 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                sizecmd
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"status\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const statushelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                status
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"struct\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const structhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setstruct
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"system\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const systemhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                syst as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"sunique\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const suniquehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                setsunique
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"tenex\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const tenexhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                settenex
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"trace\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const tracehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                settrace
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"type\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const typehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                settype
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"user\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const userhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                user as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: b"umask\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const umaskhelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 1 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                do_umask
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"verbose\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const verbosehelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 0 as ::core::ffi::c_char,
            c_handler: Some(
                setverbose
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const ::core::ffi::c_char,
                    ) -> (),
            ),
        },
        cmd {
            c_name: b"?\0" as *const u8 as *const ::core::ffi::c_char,
            c_help: &raw const helphelp as *mut ::core::ffi::c_char,
            c_bell: 0 as ::core::ffi::c_char,
            c_conn: 0 as ::core::ffi::c_char,
            c_proxy: 1 as ::core::ffi::c_char,
            c_handler: Some(
                help as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const ::core::ffi::c_char,
                ) -> (),
            ),
        },
        cmd {
            c_name: ::core::ptr::null::<::core::ffi::c_char>(),
            c_help: ::core::ptr::null::<::core::ffi::c_char>(),
            c_bell: 0,
            c_conn: 0,
            c_proxy: 0,
            c_handler: None,
        },
    ]
};

#[no_mangle]
pub static mut NCMDS: ::core::ffi::c_int = (::core::mem::size_of::<[cmd; 73]>() as usize)
        .wrapping_div(::core::mem::size_of::<cmd>() as usize)
        .wrapping_sub(1 as usize) as ::core::ffi::c_int;

pub static mut sccsid: *mut i8 = b"@(#)cmdtab.c\t5.9 (Berkeley) 3/21/89\0".as_ptr() as *mut i8;
