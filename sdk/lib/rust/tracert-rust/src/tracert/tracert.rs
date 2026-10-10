use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn _errno() -> *mut i32;
}
// ---- file-scope preamble (translated) ----
pub const SIZEOF_ICMP_ERROR: usize = 8;

pub const SIZEOF_IO_STATUS_BLOCK: usize = 8;

pub const PACKET_SIZE: usize = 32;

pub const MAX_IPADDRESS: usize = 32;

pub const NUM_OF_PINGS: u32 = 3;

pub const MIN_HOP_COUNT: u32 = 1;

pub const MAX_HOP_COUNT: u32 = 255;

pub const MIN_MILLISECONDS: u32 = 1;

pub const MAX_MILLISECONDS: u32 = u32::MAX; // ULONG_MAX

// NI_MAXHOST (ws2tcpip.h)
pub const NI_MAXHOST: usize = 1025;

#[repr(C)]
pub struct TraceInfo {
    pub ResolveAddresses: bool,
    pub MaxHops: ULONG,
    pub Timeout: ULONG,
    pub HostName: [WCHAR; NI_MAXHOST],
    pub TargetIP: [WCHAR; MAX_IPADDRESS],
    pub Family: i32,

    pub hIcmpFile: HANDLE,
    pub Target: PADDRINFOW,
}

pub type PTraceInfo = *mut TraceInfo;

pub static mut Info: TraceInfo = TraceInfo {
    ResolveAddresses: false,
    MaxHops: 0,
    Timeout: 0,
    HostName: [0; NI_MAXHOST],
    TargetIP: [0; MAX_IPADDRESS],
    Family: 0,
    hIcmpFile: core::ptr::null_mut(),
    Target: core::ptr::null_mut(),
};
unsafe extern "C" fn Usage() {
    OutputText!(IDS_USAGE);
}

unsafe extern "C" fn GetULONG(String: PCWSTR, Value: PULONG) -> bool {
    let mut StopString: PWSTR = core::ptr::null_mut();

    // Check input arguments
    if *String == UNICODE_NULL {
        return false;
    }

    // Clear errno so we can use its value after
    // the call to wcstoul to check for errors.
    *_errno() = 0;

    // Try to convert String to ULONG
    *Value = wcstoul(String, core::ptr::addr_of_mut!(StopString), 10);
    if ((*_errno()) as u32 != ERANGE) && (*_errno() != 0 || *StopString != UNICODE_NULL) {
        return false;
    }
    // The conversion was successful
    true
}

unsafe extern "C" fn ResolveTarget() -> bool {
    let mut Hints: ADDRINFOW = core::mem::zeroed();
    ZeroMemory!(
        core::ptr::addr_of_mut!(Hints) as *mut core::ffi::c_void,
        core::mem::size_of::<ADDRINFOW>(),
    );
    Hints.ai_family = Info.Family;
    Hints.ai_flags = (AI_CANONNAME as i32);

    let mut Status: i32 = 0;
    Status = GetAddrInfoW(
        core::ptr::addr_of!(Info.HostName) as PCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Hints),
        core::ptr::addr_of_mut!(Info.Target),
    );
    if Status != 0 {
        return false;
    }

    Status = GetNameInfoW(
        (*Info.Target).ai_addr,
        (*Info.Target).ai_addrlen as socklen_t,
        core::ptr::addr_of_mut!(Info.TargetIP) as PWSTR,
        MAX_IPADDRESS as DWORD,
        core::ptr::null_mut(),
        0,
        NI_NUMERICHOST as INT,
    );
    return Status == 0;
}

unsafe extern "C" fn PrintHopInfo(Buffer: PVOID) -> bool {
    let mut SockAddrIn6: SOCKADDR_IN6 = core::mem::zeroed();
    let mut SockAddrIn: SOCKADDR_IN = core::mem::zeroed();
    let mut SockAddr: PSOCKADDR;
    let mut Size: socklen_t;

    if Info.Family == (AF_INET6 as i32) {
        let Ipv6Addr: PIPV6_ADDRESS_EX = Buffer as PIPV6_ADDRESS_EX;
        SockAddrIn6.sin6_family = (AF_INET6 as i16);
        CopyMemory!(
            core::ptr::addr_of_mut!(SockAddrIn6.sin6_addr.u.Word) as *mut _,
            core::ptr::addr_of!((*Ipv6Addr).sin6_addr) as *const _,
            core::mem::size_of_val(&SockAddrIn6.sin6_addr),
        );
        //SockAddrIn6.sin6_addr = Ipv6Addr->sin6_addr;
        SockAddr = core::ptr::addr_of_mut!(SockAddrIn6) as PSOCKADDR;
        Size = core::mem::size_of::<SOCKADDR_IN6>() as socklen_t;
    } else {
        let Address: *mut IPAddr = Buffer as *mut IPAddr;
        SockAddrIn.sin_family = (AF_INET as i16);
        SockAddrIn.sin_addr.S_un.S_addr = *Address;
        SockAddr = core::ptr::addr_of_mut!(SockAddrIn) as PSOCKADDR;
        Size = core::mem::size_of::<SOCKADDR_IN>() as socklen_t;
    }

    let mut Status: INT;
    let mut Resolved: bool = false;
    let mut HostName: [WCHAR; NI_MAXHOST] = core::mem::zeroed();
    if Info.ResolveAddresses {
        Status = GetNameInfoW(
            SockAddr,
            Size,
            HostName.as_mut_ptr(),
            NI_MAXHOST as DWORD,
            core::ptr::null_mut(),
            0,
            NI_NAMEREQD as INT,
        );
        if Status == 0 {
            Resolved = true;
        }
    }

    let mut IpAddress: [WCHAR; MAX_IPADDRESS] = core::mem::zeroed();
    Status = GetNameInfoW(
        SockAddr,
        Size,
        IpAddress.as_mut_ptr(),
        MAX_IPADDRESS as DWORD,
        core::ptr::null_mut(),
        0,
        NI_NUMERICHOST as INT,
    );
    if Status == 0 {
        if Resolved {
            OutputText!(IDS_HOP_RES_INFO, HostName.as_mut_ptr(), IpAddress.as_mut_ptr());
        } else {
            OutputText!(IDS_HOP_IP_INFO, IpAddress.as_mut_ptr());
        }
    }

    return Status == 0;
}

unsafe extern "C" fn GetResponseStats(
    ReplyBuffer: PVOID,
    RoundTripTime: *mut ULONG,
    AddressInfo: *mut PVOID,
) -> ULONG {
    let mut Status: ULONG;

    if (*core::ptr::addr_of!(Info)).Family == (AF_INET6 as i32) {
        let mut EchoReplyV6: PICMPV6_ECHO_REPLY;
        EchoReplyV6 = ReplyBuffer as PICMPV6_ECHO_REPLY;
        Status = (*EchoReplyV6).Status;
        *RoundTripTime = (*EchoReplyV6).RoundTripTime;
        *AddressInfo = core::ptr::addr_of_mut!((*EchoReplyV6).Address) as PVOID;
    } else {
        let mut EchoReplyV4: PICMP_ECHO_REPLY;
        EchoReplyV4 = ReplyBuffer as PICMP_ECHO_REPLY;
        Status = (*EchoReplyV4).Status;
        *RoundTripTime = (*EchoReplyV4).RoundTripTime;
        *AddressInfo = core::ptr::addr_of_mut!((*EchoReplyV4).Address) as PVOID;
    }

    return Status;
}

unsafe extern "C" fn DecodeResponse(
    ReplyBuffer: *mut core::ffi::c_void,
    LastGoodResponse: *mut core::ffi::c_void,
    OutputHopAddress: bool,
    GoodResponse: *mut bool,
    FoundTarget: *mut bool,
) -> bool {
    let mut RoundTripTime: u32 = 0;
    let mut AddressInfo: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: u32 = GetResponseStats(ReplyBuffer, &mut RoundTripTime, &raw mut AddressInfo);

    match Status {
        IP_SUCCESS | IP_TTL_EXPIRED_TRANSIT => {
            if RoundTripTime != 0 {
                OutputText!(IDS_HOP_TIME, RoundTripTime);
            } else {
                OutputText!(IDS_HOP_ZERO);
            }
            *GoodResponse = true;
        }

        IP_DEST_HOST_UNREACHABLE | IP_DEST_NET_UNREACHABLE => {
            *FoundTarget = true;
            PrintHopInfo(AddressInfo);
            OutputText!(IDS_HOP_RESPONSE);
            if Status == IP_DEST_HOST_UNREACHABLE {
                OutputText!(IDS_DEST_HOST_UNREACHABLE);
            } else if Status == IP_DEST_NET_UNREACHABLE {
                OutputText!(IDS_DEST_NET_UNREACHABLE);
            }
            return true;
        }

        IP_REQ_TIMED_OUT => {
            OutputText!(IDS_TIMEOUT);
        }

        IP_GENERAL_FAILURE => {
            OutputText!(IDS_GEN_FAILURE);
            return false;
        }

        _ => {
            OutputText!(IDS_TRANSMIT_FAILED, Status);
            return false;
        }
    }

    if OutputHopAddress {
        if Status == IP_REQ_TIMED_OUT && !LastGoodResponse.is_null() {
            Status = GetResponseStats(LastGoodResponse, &mut RoundTripTime, &raw mut AddressInfo);
        }
        if Status == IP_SUCCESS {
            *FoundTarget = true;
        }
        if Status == IP_TTL_EXPIRED_TRANSIT || Status == IP_SUCCESS {
            PrintHopInfo(AddressInfo);
            OutputText!(IDS_LINEBREAK);
        } else if Status == IP_REQ_TIMED_OUT {
            OutputText!(IDS_REQ_TIMED_OUT);
        }
    }

    true
}

unsafe extern "C" fn RunTraceRoute() -> bool {
    let mut Success: bool = false;
    let mut ReplyBuffer: PVOID = core::ptr::null_mut();
    let mut LastGoodResponse: PVOID = core::ptr::null_mut();
    let mut ReplySize: DWORD;

    let heap: HANDLE = GetProcessHeap();
    let mut Quit: bool = false;
    let mut HopCount: ULONG = 1;
    let mut FoundTarget: bool = false;

    'Cleanup: {
        Success = ResolveTarget();
        if !Success {
            OutputText!(IDS_UNABLE_RESOLVE, addr_of_mut!(Info.HostName) as *mut WCHAR);
            break 'Cleanup;
        }

        ReplySize = (PACKET_SIZE + SIZEOF_ICMP_ERROR + SIZEOF_IO_STATUS_BLOCK) as _;
        if Info.Family == (AF_INET6 as i32) {
            ReplySize += core::mem::size_of::<ICMPV6_ECHO_REPLY>() as DWORD;
        } else {
            ReplySize += core::mem::size_of::<ICMP_ECHO_REPLY>() as DWORD;
        }

        ReplyBuffer = HeapAlloc(heap, HEAP_ZERO_MEMORY, ReplySize as SIZE_T);
        if ReplyBuffer.is_null() {
            Success = false;
            break 'Cleanup;
        }

        if Info.Family == (AF_INET6 as i32) {
            Info.hIcmpFile = Icmp6CreateFile();
        } else {
            Info.hIcmpFile = IcmpCreateFile();
        }
        if Info.hIcmpFile == INVALID_HANDLE_VALUE {
            Success = false;
            break 'Cleanup;
        }

        OutputText!(
            IDS_TRACE_INFO,
            addr_of_mut!(Info.HostName) as *mut WCHAR,
            addr_of_mut!(Info.TargetIP) as *mut WCHAR,
            Info.MaxHops,
        );

        let mut IpOptionInfo: IP_OPTION_INFORMATION = core::mem::zeroed();
        ZeroMemory!(
            core::ptr::addr_of_mut!(IpOptionInfo) as PVOID,
            core::mem::size_of::<IP_OPTION_INFORMATION>(),
        );

        while (HopCount <= Info.MaxHops) && (FoundTarget == false) && (Quit == false) {
            OutputText!(IDS_HOP_COUNT, HopCount);

            if !LastGoodResponse.is_null() {
                HeapFree(heap, 0, LastGoodResponse);
                LastGoodResponse = core::ptr::null_mut();
            }

            let mut Ping: i32 = 1;
            while Ping <= (NUM_OF_PINGS as i32) {
                let mut SendBuffer: [BYTE; PACKET_SIZE] = [0; PACKET_SIZE];
                let mut GoodResponse: bool = false;

                IpOptionInfo.Ttl = HopCount as UCHAR;

                if Info.Family == (AF_INET6 as i32) {
                    let mut Source: sockaddr_in6 = core::mem::zeroed();

                    ZeroMemory!(
                        core::ptr::addr_of_mut!(Source) as PVOID,
                        core::mem::size_of::<sockaddr_in6>(),
                    );
                    Source.sin6_family = (AF_INET6 as u16);

                    let _ = Icmp6SendEcho2(
                        Info.hIcmpFile,
                        core::ptr::null_mut(),
                        None, // ApcRoutine (FARPROC)
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(Source),
                        (*Info.Target).ai_addr as *mut sockaddr_in6,
                        SendBuffer.as_mut_ptr() as LPVOID,
                        PACKET_SIZE as USHORT,
                        core::ptr::addr_of_mut!(IpOptionInfo),
                        ReplyBuffer,
                        ReplySize,
                        Info.Timeout,
                    );
                } else {
                    let _ = IcmpSendEcho2(
                        Info.hIcmpFile,
                        core::ptr::null_mut(),
                        None, // ApcRoutine (FARPROC)
                        core::ptr::null_mut(),
                        (*((*Info.Target).ai_addr as PSOCKADDR_IN)).sin_addr.S_un.S_addr,
                        SendBuffer.as_mut_ptr() as LPVOID,
                        PACKET_SIZE as USHORT,
                        core::ptr::addr_of_mut!(IpOptionInfo),
                        ReplyBuffer,
                        ReplySize,
                        Info.Timeout,
                    );
                }

                if DecodeResponse(
                    ReplyBuffer,
                    LastGoodResponse,
                    Ping == (NUM_OF_PINGS as i32),
                    core::ptr::addr_of_mut!(GoodResponse),
                    core::ptr::addr_of_mut!(FoundTarget),
                ) == false
                {
                    Quit = true;
                    break;
                }

                if FoundTarget {
                    Success = true;
                    break;
                }

                if GoodResponse {
                    if !LastGoodResponse.is_null() {
                        HeapFree(heap, 0, LastGoodResponse);
                    }
                    LastGoodResponse = HeapAlloc(heap, HEAP_ZERO_MEMORY, ReplySize as SIZE_T);
                    if LastGoodResponse.is_null() {
                        Success = false;
                        break 'Cleanup;
                    }
                    CopyMemory!(LastGoodResponse, ReplyBuffer, ReplySize);
                }

                Ping += 1;
            }

            HopCount += 1;
            Sleep(100);
        }

        OutputText!(IDS_TRACE_COMPLETE);
    }

    // Cleanup:
    if !ReplyBuffer.is_null() {
        HeapFree(heap, 0, ReplyBuffer);
    }
    if !LastGoodResponse.is_null() {
        HeapFree(heap, 0, LastGoodResponse);
    }
    if !Info.Target.is_null() {
        FreeAddrInfoW(Info.Target);
    }
    if !Info.hIcmpFile.is_null() {
        IcmpCloseHandle(Info.hIcmpFile);
    }

    Success
}

unsafe extern "C" fn GetUlongOptionInRange(
    argc: i32,
    argv: *mut *mut wchar_t,
    i: *mut i32,
    Value: *mut ULONG,
    MinimumValue: ULONG,
    MaximumValue: ULONG,
) -> bool {
    let mut ParsedValue: ULONG = 0;

    // Check if we have enough values
    if (*i + 1) > (argc - 1) {
        OutputText!(IDS_MISSING_OPTION_VALUE, *argv.offset(*i as isize));
        return false;
    }

    *i += 1;

    // Try to parse and convert the value as ULONG.
    // Check if ParsedValue is within the specified range.
    if !GetULONG(
        *argv.offset(*i as isize) as PCWSTR,
        core::ptr::addr_of_mut!(ParsedValue),
    ) || ((ParsedValue < MinimumValue) || (ParsedValue > MaximumValue))
    {
        *i -= 1;
        OutputText!(IDS_BAD_OPTION_VALUE, *argv.offset(*i as isize));
        return false;
    }

    *Value = ParsedValue;
    return true;
}

unsafe extern "C" fn ParseCmdline(argc: i32, argv: *mut *mut wchar_t) -> bool {
    if argc < 2 as i32 {
        Usage();
        return false;
    }

    let mut i: i32 = 1 as i32;
    while i < argc {
        if *(*argv.add(i as usize)).add(0 as usize) == '-' as wchar_t
            || *(*argv.add(i as usize)).add(0 as usize) == '/' as wchar_t
        {
            let c: wchar_t = *(*argv.add(i as usize)).add(1 as usize);
            match c {
                0x64 => {
                    // 'd'
                    Info.ResolveAddresses = false;
                }
                0x68 => {
                    // 'h'
                    if !GetUlongOptionInRange(
                        argc,
                        argv,
                        core::ptr::addr_of_mut!(i),
                        core::ptr::addr_of_mut!(Info.MaxHops),
                        MIN_HOP_COUNT as u32,
                        MAX_HOP_COUNT as u32,
                    ) {
                        return false;
                    }
                }
                0x6A => {
                    // 'j'
                    printf(b"-j is not yet implemented.\n\0".as_ptr() as *const i8);
                    return false;
                }
                0x77 => {
                    // 'w'
                    if !GetUlongOptionInRange(
                        argc,
                        argv,
                        core::ptr::addr_of_mut!(i),
                        core::ptr::addr_of_mut!(Info.Timeout),
                        MIN_MILLISECONDS as u32,
                        MAX_MILLISECONDS as u32,
                    ) {
                        return false;
                    }
                }
                0x34 => {
                    // '4'
                    Info.Family = AF_INET as i32;
                }
                0x36 => {
                    // '6'
                    Info.Family = AF_INET6 as i32;
                }
                _ => {
                    OutputText!(IDS_INVALID_OPTION, *argv.add(i as usize));
                    Usage();
                    return false;
                }
            }
        } else {
            // The host must be the last argument
            if i != (argc - 1 as i32) {
                Usage();
                return false;
            }

            StringCchCopyW(
                core::ptr::addr_of_mut!(Info.HostName) as *mut wchar_t,
                NI_MAXHOST as usize,
                *argv.add(i as usize),
            );
            break;
        }
        i += 1;
    }

    // Check for missing host
    if Info.HostName[0 as usize] == UNICODE_NULL as wchar_t {
        OutputText!(IDS_MISSING_TARGET);
        Usage();
        return false;
    }
    return true;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: i32, argv: *mut *mut wchar_t) -> i32 {
    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    (*core::ptr::addr_of_mut!(Info)).ResolveAddresses = true;
    (*core::ptr::addr_of_mut!(Info)).MaxHops = 30;
    (*core::ptr::addr_of_mut!(Info)).Timeout = 4000;
    (*core::ptr::addr_of_mut!(Info)).Family = (AF_UNSPEC as i32);

    if !ParseCmdline(argc, argv) {
        return 1;
    }

    let mut WsaData: WSADATA = core::mem::zeroed();
    if WSAStartup(MAKEWORD!(2, 2), &mut WsaData) != 0 {
        return 1;
    }

    let mut Success: bool;
    Success = RunTraceRoute();

    WSACleanup();

    if Success { 0 } else { 1 }
}
