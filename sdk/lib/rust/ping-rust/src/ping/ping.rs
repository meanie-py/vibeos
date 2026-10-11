// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_2D1785B5C90E: [u16; 5] = [0x0049u16, 0x0050, 0x0076, 0x0036, 0x0000];
pub static RUSTCONV_WSTR_AE3EDD5AD3C1: [u16; 5] = [0x0049u16, 0x0050, 0x0076, 0x0034, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut hIcmpFile: HANDLE = unsafe { INVALID_HANDLE_VALUE };

static mut Timeout: ULONG = 4000 as ULONG;

static mut Family: ::core::ffi::c_int = (AF_UNSPEC as i32);

static mut RequestSize: ULONG = 32 as ULONG;

static mut PingCount: ULONG = 4 as ULONG;

static mut PingForever: BOOL = (FALSE) as i32;

static mut Target: PADDRINFOW = ::core::ptr::null::<addrinfoW>() as *mut addrinfoW;

static mut TargetName: PCWSTR = ::core::ptr::null::<WCHAR>();

static mut Address: [WCHAR; 46] = [0; 46];

static mut CanonName: [WCHAR; 1025] = [0; 1025];

static mut ResolveAddress: BOOL = (FALSE) as i32;

static mut RTTMax: ULONG = 0 as ULONG;

static mut RTTMin: ULONG = 0 as ULONG;

static mut RTTTotal: ULONG = 0 as ULONG;

static mut EchosSent: ULONG = 0 as ULONG;

static mut EchosReceived: ULONG = 0 as ULONG;

static mut EchosSuccessful: ULONG = 0 as ULONG;

static mut IpOptions: IP_OPTION_INFORMATION = ip_option_information {
    Ttl: 0,
    Tos: 0,
    Flags: 0,
    OptionsSize: 0,
    OptionsData: ::core::ptr::null::<::core::ffi::c_uchar>() as *mut ::core::ffi::c_uchar,
};

// WIN32_LEAN_AND_MEAN, WIN32_NO_STATUS, NDEBUG: flag-only defines with no value, omitted.

pub const SIZEOF_ICMP_ERROR: usize = 8;
pub const SIZEOF_IO_STATUS_BLOCK: usize = 8;
pub const DEFAULT_TIMEOUT: ULONG = 1000;
pub const MAX_SEND_SIZE: ULONG = 65500;

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argv: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut wsaData: WSADATA = core::mem::zeroed();
    let mut i: ULONG;
    let mut StrLen: DWORD = 46;
    let mut Status: ::core::ffi::c_int;

    // PHANDLER_ROUTINE is WINAPI: stdcall on i386, the platform C ABI elsewhere. The two
    // ABIs are distinct fn-pointer types, so the handler is declared per arch.
    #[cfg(target_arch = "x86")]
    unsafe extern "stdcall" fn ctrl_handler_c(ctrl: DWORD) -> BOOL {
        ConsoleCtrlHandler(ctrl)
    }
    #[cfg(not(target_arch = "x86"))]
    unsafe extern "C" fn ctrl_handler_c(ctrl: DWORD) -> BOOL {
        ConsoleCtrlHandler(ctrl)
    }

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    IpOptions.Ttl = 128;

    if ParseCmdLine(argc, argv) == 0 {
        return 1;
    }

    if SetConsoleCtrlHandler(Some(ctrl_handler_c), TRUE as ::core::ffi::c_int) == 0 {
        DPRINT!("Failed to set control handler: %lu\n", GetLastError());
        return 1;
    }

    Status = WSAStartup(MAKEWORD!(2, 2), core::ptr::addr_of_mut!(wsaData));
    if Status != 0 {
        ConResPrintf(StdErr, IDS_WINSOCK_FAIL, Status);
        return 1;
    }

    if ResolveTarget(TargetName) == 0 {
        WSACleanup();
        return 1;
    }

    if WSAAddressToStringW(
        (*Target).ai_addr as LPSOCKADDR,
        (*Target).ai_addrlen as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Address) as LPWSTR,
        core::ptr::addr_of_mut!(StrLen),
    ) != 0
    {
        DPRINT!("WSAAddressToStringW failed: %d\n", WSAGetLastError());
        FreeAddrInfoW(Target);
        WSACleanup();
        return 1;
    }

    if Family == (AF_INET6 as i32) {
        hIcmpFile = Icmp6CreateFile();
    } else {
        hIcmpFile = IcmpCreateFile();
    }

    if hIcmpFile == INVALID_HANDLE_VALUE {
        DPRINT!("IcmpCreateFile failed: %lu\n", GetLastError());
        FreeAddrInfoW(Target);
        WSACleanup();
        return 1;
    }

    if *(core::ptr::addr_of_mut!(CanonName) as *mut WCHAR) != 0 {
        ConResPrintf(
            StdOut,
            IDS_PINGING_HOSTNAME,
            core::ptr::addr_of_mut!(CanonName) as *mut WCHAR,
            core::ptr::addr_of_mut!(Address) as *mut WCHAR,
        );
    } else {
        ConResPrintf(
            StdOut,
            IDS_PINGING_ADDRESS,
            core::ptr::addr_of_mut!(Address) as *mut WCHAR,
        );
    }

    ConResPrintf(StdOut, IDS_PING_SIZE, RequestSize);

    Ping();

    i = 1;
    while i < PingCount {
        Sleep(1000);
        Ping();

        if PingForever == 0 {
            i = i.wrapping_add(1);
        }
    }

    PrintStats();

    IcmpCloseHandle(hIcmpFile);
    FreeAddrInfoW(Target);
    WSACleanup();

    return 0;
}

unsafe extern "C" fn ParseCmdLine(argc: i32, argv: *mut PWSTR) -> BOOL {
    let mut i: i32;
    if argc < 2 {
        ConResPrintf(&raw mut csStdOut, IDS_USAGE);
        return (FALSE) as BOOL;
    }
    i = 1;
    while i < argc {
        if *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Du16
            || *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16
        {
            let sw: u16 = *(*argv.offset(i as isize)).offset(1 as isize);
            if sw == 0x0074u16 {
                PingForever = (TRUE) as i32;
            } else if sw == 0x0061u16 {
                ResolveAddress = (TRUE) as i32;
            } else if sw == 0x006Eu16 {
                if i + 1 < argc {
                    PingForever = (FALSE) as i32;
                    i += 1;
                    PingCount = wcstoul(
                        *argv.offset(i as isize) as *const u16,
                        core::ptr::null_mut(),
                        0,
                    ) as ULONG;
                    if PingCount == 0 {
                        ConResPrintf(
                            &raw mut csStdErr,
                            IDS_BAD_VALUE,
                            *argv.offset((i - 1) as isize),
                            1,
                            UINT_MAX,
                        );
                        return (FALSE) as BOOL;
                    }
                } else {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_MISSING_VALUE,
                        *argv.offset(i as isize),
                    );
                    return (FALSE) as BOOL;
                }
            } else if sw == 0x006Cu16 {
                if i + 1 < argc {
                    i += 1;
                    RequestSize = wcstoul(
                        *argv.offset(i as isize) as *const u16,
                        core::ptr::null_mut(),
                        0,
                    ) as ULONG;
                    if RequestSize > MAX_SEND_SIZE {
                        ConResPrintf(
                            &raw mut csStdErr,
                            IDS_BAD_VALUE,
                            *argv.offset((i - 1) as isize),
                            0,
                            MAX_SEND_SIZE,
                        );
                        return (FALSE) as BOOL;
                    }
                } else {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_MISSING_VALUE,
                        *argv.offset(i as isize),
                    );
                    return (FALSE) as BOOL;
                }
            } else if sw == 0x0066u16 {
                if Family == (AF_INET6 as i32) {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_WRONG_FAMILY,
                        *argv.offset(i as isize),
                        RUSTCONV_WSTR_AE3EDD5AD3C1.as_ptr(),
                    );
                    return (FALSE) as BOOL;
                }
                Family = (AF_INET as i32);
                IpOptions.Flags |= (IP_FLAG_DF as u8);
            } else if sw == 0x0069u16 {
                if i + 1 < argc {
                    i += 1;
                    let Ttl: ULONG = wcstoul(
                        *argv.offset(i as isize) as *const u16,
                        core::ptr::null_mut(),
                        0,
                    ) as ULONG;

                    if (Ttl == 0) || (Ttl > UCHAR_MAX) {
                        ConResPrintf(
                            &raw mut csStdErr,
                            IDS_BAD_VALUE,
                            *argv.offset((i - 1) as isize),
                            1,
                            UCHAR_MAX,
                        );
                        return (FALSE) as BOOL;
                    }

                    IpOptions.Ttl = Ttl as UCHAR;
                } else {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_MISSING_VALUE,
                        *argv.offset(i as isize),
                    );
                    return (FALSE) as BOOL;
                }
            } else if sw == 0x0076u16 {
                if Family == (AF_INET6 as i32) {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_WRONG_FAMILY,
                        *argv.offset(i as isize),
                        RUSTCONV_WSTR_AE3EDD5AD3C1.as_ptr(),
                    );
                    return (FALSE) as BOOL;
                }

                Family = (AF_INET as i32);

                if i + 1 < argc {
                    /* This option has been deprecated. Don't do anything. */
                    i += 1;
                } else {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_MISSING_VALUE,
                        *argv.offset(i as isize),
                    );
                    return (FALSE) as BOOL;
                }
            } else if sw == 0x0077u16 {
                if i + 1 < argc {
                    i += 1;
                    Timeout = wcstoul(
                        *argv.offset(i as isize) as *const u16,
                        core::ptr::null_mut(),
                        0,
                    ) as ULONG;
                    if Timeout < DEFAULT_TIMEOUT {
                        Timeout = DEFAULT_TIMEOUT;
                    }
                } else {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_MISSING_VALUE,
                        *argv.offset(i as isize),
                    );
                    return (FALSE) as BOOL;
                }
            } else if sw == 0x0052u16 {
                if Family == (AF_INET as i32) {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_WRONG_FAMILY,
                        *argv.offset(i as isize),
                        RUSTCONV_WSTR_2D1785B5C90E.as_ptr(),
                    );
                    return (FALSE) as BOOL;
                }

                Family = (AF_INET6 as i32);

                /* This option has been deprecated. Don't do anything. */
            } else if sw == 0x0034u16 {
                if Family == (AF_INET6 as i32) {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_WRONG_FAMILY,
                        *argv.offset(i as isize),
                        RUSTCONV_WSTR_AE3EDD5AD3C1.as_ptr(),
                    );
                    return (FALSE) as BOOL;
                }

                Family = (AF_INET as i32);
            } else if sw == 0x0036u16 {
                if Family == (AF_INET as i32) {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_WRONG_FAMILY,
                        *argv.offset(i as isize),
                        RUSTCONV_WSTR_2D1785B5C90E.as_ptr(),
                    );
                    return (FALSE) as BOOL;
                }

                Family = (AF_INET6 as i32);
            } else if sw == 0x003Fu16 {
                ConResPrintf(&raw mut csStdOut, IDS_USAGE);
                return (FALSE) as BOOL;
            } else {
                ConResPrintf(
                    &raw mut csStdErr,
                    IDS_BAD_OPTION,
                    *argv.offset(i as isize),
                );
                ConResPrintf(&raw mut csStdErr, IDS_USAGE);
                return (FALSE) as BOOL;
            }
        } else {
            if !TargetName.is_null() {
                ConResPrintf(
                    &raw mut csStdErr,
                    IDS_BAD_PARAMETER,
                    *argv.offset(i as isize),
                );
                return (FALSE) as BOOL;
            }

            TargetName = *argv.offset(i as isize) as PCWSTR;
        }
        i += 1;
    }

    if TargetName.is_null() {
        ConResPrintf(&raw mut csStdErr, IDS_MISSING_ADDRESS);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ResolveTarget(target: PCWSTR) -> BOOL {
    let mut hints: ADDRINFOW = core::mem::zeroed();
    let mut Status: i32;

    ZeroMemory!(
        core::ptr::addr_of_mut!(hints) as *mut core::ffi::c_void,
        core::mem::size_of::<ADDRINFOW>(),
    );
    hints.ai_family = Family;
    hints.ai_flags = (AI_NUMERICHOST as i32);

    Status = GetAddrInfoW(
        target,
        core::ptr::null(),
        core::ptr::addr_of_mut!(hints),
        core::ptr::addr_of_mut!(Target),
    );
    if Status != 0 {
        hints.ai_flags = (AI_CANONNAME as i32);

        Status = GetAddrInfoW(
            target,
            core::ptr::null(),
            core::ptr::addr_of_mut!(hints),
            core::ptr::addr_of_mut!(Target),
        );
        if Status != 0 {
            ConResPrintf(
                core::ptr::addr_of_mut!(csStdOut),
                IDS_UNKNOWN_HOST,
                target,
            );
            return (FALSE) as BOOL;
        }

        wcsncpy(
            core::ptr::addr_of_mut!(CanonName) as *mut WCHAR,
            (*Target).ai_canonname as *const WCHAR,
            wcslen((*Target).ai_canonname as *const WCHAR) as usize,
        );
    } else if ResolveAddress != 0 {
        Status = GetNameInfoW(
            (*Target).ai_addr,
            ((*Target).ai_addrlen as u32) as i32,
            core::ptr::addr_of_mut!(CanonName) as PWCHAR,
            _countof!(CanonName) as DWORD,
            core::ptr::null_mut(),
            0,
            (NI_NAMEREQD) as INT,
        );
        if Status != 0 {
            DPRINT!("GetNameInfoW failed: %d\n", WSAGetLastError());
        }
    }

    Family = (*Target).ai_family;

    return (TRUE) as BOOL;
}

unsafe extern "C" fn Ping() {
    let mut ReplyBuffer: PVOID = core::ptr::null_mut();
    let mut SendBuffer: PVOID = core::ptr::null_mut();
    let mut ReplySize: DWORD = 0;
    let mut Status: DWORD = 0;

    if RequestSize != 0 {
        SendBuffer = malloc(RequestSize as size_t) as PVOID;
        if SendBuffer.is_null() {
            ConResPrintf(StdErr, IDS_NO_RESOURCES as UINT);
            exit(1 as ::core::ffi::c_int);
        }

        /* Windows ping utility fills the optional data field with
         * ASCII characters from 'a' to 'w', wrapping back around
         * until SendBuffer is full. */
        let mut i: ULONG = 0;
        while i < RequestSize {
            *(SendBuffer as PUCHAR).offset(i as isize) =
                (('a' as ULONG) + (i % (('w' as ULONG) - ('a' as ULONG) + 1))) as UCHAR;
            i = i.wrapping_add(1);
        }
    }

    if Family == (AF_INET6 as i32) {
        ReplySize = ReplySize.wrapping_add(core::mem::size_of::<ICMPV6_ECHO_REPLY>() as DWORD);
    } else {
        if cfg!(target_pointer_width = "64") {
            ReplySize = ReplySize.wrapping_add(core::mem::size_of::<ICMP_ECHO_REPLY>() as DWORD);
        } else {
            ReplySize = ReplySize.wrapping_add(core::mem::size_of::<ICMP_ECHO_REPLY>() as DWORD);
        }
    }

    ReplySize = ReplySize.wrapping_add(RequestSize + (SIZEOF_ICMP_ERROR as u32) + (SIZEOF_IO_STATUS_BLOCK as u32));

    ReplyBuffer = malloc(ReplySize as size_t) as PVOID;
    if ReplyBuffer.is_null() {
        ConResPrintf(StdErr, IDS_NO_RESOURCES as UINT);
        free(SendBuffer as *mut ::core::ffi::c_void);
        exit(1 as ::core::ffi::c_int);
    }

    ZeroMemory!(ReplyBuffer, ReplySize as size_t);

    EchosSent = EchosSent.wrapping_add(1);

    if Family == (AF_INET6 as i32) {
        let mut Source: sockaddr_in6 = core::mem::zeroed();

        ZeroMemory!(core::ptr::addr_of_mut!(Source) as PVOID, core::mem::size_of::<sockaddr_in6>() as size_t);
        Source.sin6_family = AF_INET6 as ADDRESS_FAMILY;

        Status = Icmp6SendEcho2(
            hIcmpFile,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Source),
            (*Target).ai_addr as *mut sockaddr_in6,
            SendBuffer as LPVOID,
            RequestSize as USHORT,
            core::ptr::addr_of_mut!(IpOptions),
            ReplyBuffer as LPVOID,
            ReplySize,
            Timeout,
        );
    } else {
        Status = IcmpSendEcho2(
            hIcmpFile,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            (*((*Target).ai_addr as PSOCKADDR_IN)).sin_addr.S_un.S_addr,
            SendBuffer as LPVOID,
            RequestSize as USHORT,
            core::ptr::addr_of_mut!(IpOptions),
            ReplyBuffer as LPVOID,
            ReplySize,
            Timeout,
        );
    }

    /* TODO: Compare ReplyBuffer data to SendBuffer. */
    free(SendBuffer as *mut ::core::ffi::c_void);

    if Status == 0 {
        Status = GetLastError();
        match Status {
            IP_REQ_TIMED_OUT => {
                ConResPrintf(StdOut, IDS_REQUEST_TIMED_OUT as UINT);
            }
            _ => {
                ConResPrintf(StdOut, IDS_TRANSMIT_FAILED as UINT, Status);
            }
        }
    } else {
        let mut SockAddrIn6: SOCKADDR_IN6 = core::mem::zeroed();
        let mut SockAddrIn: SOCKADDR_IN = core::mem::zeroed();
        let mut SockAddr: PSOCKADDR = core::ptr::null_mut();
        let mut Size: socklen_t = 0;

        EchosReceived = EchosReceived.wrapping_add(1);

        ZeroMemory!(core::ptr::addr_of_mut!(SockAddrIn) as PVOID, core::mem::size_of::<SOCKADDR_IN>() as size_t);
        ZeroMemory!(core::ptr::addr_of_mut!(SockAddrIn6) as PVOID, core::mem::size_of::<SOCKADDR_IN6>() as size_t);

        if Family == (AF_INET6 as i32) {
            let mut pEchoReply: PICMPV6_ECHO_REPLY = core::ptr::null_mut();
            let mut Ipv6Addr: PIPV6_ADDRESS_EX = core::ptr::null_mut();

            pEchoReply = ReplyBuffer as PICMPV6_ECHO_REPLY;

            Ipv6Addr = core::ptr::addr_of_mut!((*pEchoReply).Address) as PIPV6_ADDRESS_EX;
            SockAddrIn6.sin6_family = (AF_INET6 as ADDRESS_FAMILY) as i16;
            CopyMemory!(
                core::ptr::addr_of_mut!(SockAddrIn6.sin6_addr.u.Word) as PVOID,
                core::ptr::addr_of!((*Ipv6Addr).sin6_addr) as *const ::core::ffi::c_void,
                core::mem::size_of::<in6_addr>() as size_t,
            );
            //SockAddrIn6.sin6_addr = Ipv6Addr->sin6_addr;
            SockAddr = core::ptr::addr_of_mut!(SockAddrIn6) as PSOCKADDR;
            Size = core::mem::size_of::<SOCKADDR_IN6>() as socklen_t;

            GetNameInfoW(
                SockAddr,
                Size,
                core::ptr::addr_of_mut!(Address) as PWCHAR,
                NI_MAXHOST as DWORD,
                core::ptr::null_mut(),
                0 as DWORD,
                NI_NUMERICHOST as ::core::ffi::c_int,
            );

            ConResPrintf(StdOut, IDS_REPLY_FROM as UINT, core::ptr::addr_of_mut!(Address) as *mut WCHAR);

            match (*pEchoReply).Status {
                IP_SUCCESS => {
                    EchosSuccessful = EchosSuccessful.wrapping_add(1);

                    if (*pEchoReply).RoundTripTime == 0 {
                        ConResPrintf(StdOut, IDS_REPLY_TIME_0MS as UINT);
                    } else {
                        ConResPrintf(StdOut, IDS_REPLY_TIME_MS as UINT, (*pEchoReply).RoundTripTime);
                    }

                    if (*pEchoReply).RoundTripTime < RTTMin || RTTMin == 0 {
                        RTTMin = (*pEchoReply).RoundTripTime;
                    }

                    if (*pEchoReply).RoundTripTime > RTTMax || RTTMax == 0 {
                        RTTMax = (*pEchoReply).RoundTripTime;
                    }

                    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

                    RTTTotal = RTTTotal.wrapping_add((*pEchoReply).RoundTripTime);
                }

                IP_DEST_NET_UNREACHABLE => {
                    ConResPrintf(StdOut, IDS_DEST_NET_UNREACHABLE as UINT);
                }

                IP_DEST_HOST_UNREACHABLE => {
                    ConResPrintf(StdOut, IDS_DEST_HOST_UNREACHABLE as UINT);
                }

                IP_TTL_EXPIRED_TRANSIT => {
                    ConResPrintf(StdOut, IDS_TTL_EXPIRED as UINT);
                }

                _ => {
                    ConResPrintf(StdOut, IDS_REPLY_STATUS as UINT, (*pEchoReply).Status);
                }
            }
        } else {
            let mut pEchoReply: PICMP_ECHO_REPLY = core::ptr::null_mut();
            let mut IP4Addr: *mut IPAddr = core::ptr::null_mut();

            pEchoReply = ReplyBuffer as PICMP_ECHO_REPLY;

            IP4Addr = core::ptr::addr_of_mut!((*pEchoReply).Address) as *mut IPAddr;
            SockAddrIn.sin_family = (AF_INET as ADDRESS_FAMILY) as i16;
            SockAddrIn.sin_addr.S_un.S_addr = *IP4Addr;
            SockAddr = core::ptr::addr_of_mut!(SockAddrIn) as PSOCKADDR;
            Size = core::mem::size_of::<SOCKADDR_IN>() as socklen_t;

            GetNameInfoW(
                SockAddr,
                Size,
                core::ptr::addr_of_mut!(Address) as PWCHAR,
                NI_MAXHOST as DWORD,
                core::ptr::null_mut(),
                0 as DWORD,
                NI_NUMERICHOST as ::core::ffi::c_int,
            );

            ConResPrintf(StdOut, IDS_REPLY_FROM as UINT, core::ptr::addr_of_mut!(Address) as *mut WCHAR);

            match (*pEchoReply).Status {
                IP_SUCCESS => {
                    EchosSuccessful = EchosSuccessful.wrapping_add(1);

                    ConResPrintf(
                        StdOut,
                        IDS_REPLY_BYTES as UINT,
                        (*pEchoReply).DataSize as ::core::ffi::c_int,
                    );

                    if (*pEchoReply).RoundTripTime == 0 {
                        ConResPrintf(StdOut, IDS_REPLY_TIME_0MS as UINT);
                    } else {
                        ConResPrintf(StdOut, IDS_REPLY_TIME_MS as UINT, (*pEchoReply).RoundTripTime);
                    }

                    ConResPrintf(
                        StdOut,
                        IDS_REPLY_TTL as UINT,
                        (*pEchoReply).Options.Ttl as ::core::ffi::c_int,
                    );

                    if (*pEchoReply).RoundTripTime < RTTMin || RTTMin == 0 {
                        RTTMin = (*pEchoReply).RoundTripTime;
                    }

                    if (*pEchoReply).RoundTripTime > RTTMax || RTTMax == 0 {
                        RTTMax = (*pEchoReply).RoundTripTime;
                    }

                    RTTTotal = RTTTotal.wrapping_add((*pEchoReply).RoundTripTime);
                }

                IP_DEST_NET_UNREACHABLE => {
                    ConResPrintf(StdOut, IDS_DEST_NET_UNREACHABLE as UINT);
                }

                IP_DEST_HOST_UNREACHABLE => {
                    ConResPrintf(StdOut, IDS_DEST_HOST_UNREACHABLE as UINT);
                }

                IP_TTL_EXPIRED_TRANSIT => {
                    ConResPrintf(StdOut, IDS_TTL_EXPIRED as UINT);
                }

                _ => {
                    ConResPrintf(StdOut, IDS_REPLY_STATUS as UINT, (*pEchoReply).Status);
                }
            }
        }
    }

    free(ReplyBuffer as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn PrintStats() {
    let mut EchosLost: ULONG = EchosSent.wrapping_sub(EchosReceived);
    let mut PercentLost: ULONG = ((EchosLost as ::core::ffi::c_double
        / EchosSent as ::core::ffi::c_double)
        * 100.0) as ULONG;
    ConResPrintf(
        &raw mut csStdOut,
        IDS_STATISTICS as UINT,
        core::ptr::addr_of_mut!(Address) as *mut WCHAR,
        EchosSent,
        EchosReceived,
        EchosLost,
        PercentLost,
    );
    if EchosSuccessful > 0 {
        let mut RTTAverage: ULONG = RTTTotal.wrapping_div(EchosSuccessful);
        ConResPrintf(
            &raw mut csStdOut,
            IDS_APPROXIMATE_RTT as UINT,
            RTTMin,
            RTTMax,
            RTTAverage,
        );
    }
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn ConsoleCtrlHandler(ControlType: DWORD) -> BOOL {
    match ControlType {
        CTRL_C_EVENT => {
            PrintStats();
            ConResPrintf(&raw mut csStdOut, IDS_CTRL_C as UINT);
            return (FALSE) as BOOL;
        }

        CTRL_BREAK_EVENT => {
            PrintStats();
            ConResPrintf(&raw mut csStdOut, IDS_CTRL_BREAK as UINT);
            return (TRUE) as BOOL;
        }

        CTRL_CLOSE_EVENT => {
            PrintStats();
            return (FALSE) as BOOL;
        }

        _ => {
            return (FALSE) as BOOL;
        }
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn ConsoleCtrlHandler(ControlType: DWORD) -> BOOL {
    match ControlType {
        CTRL_C_EVENT => {
            PrintStats();
            ConResPrintf(&raw mut csStdOut, IDS_CTRL_C as UINT);
            return (FALSE) as BOOL;
        }

        CTRL_BREAK_EVENT => {
            PrintStats();
            ConResPrintf(&raw mut csStdOut, IDS_CTRL_BREAK as UINT);
            return (TRUE) as BOOL;
        }

        CTRL_CLOSE_EVENT => {
            PrintStats();
            return (FALSE) as BOOL;
        }

        _ => {
            return (FALSE) as BOOL;
        }
    }
}
