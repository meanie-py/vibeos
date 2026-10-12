use crate::nslookup::utility::ClassNametoClassID;
use crate::nslookup::utility::ExtractIP;
use crate::nslookup::utility::ExtractName;
use crate::nslookup::utility::IsValidIP;
use crate::nslookup::utility::ReverseIP;
use crate::nslookup::utility::SendRequest;
use crate::nslookup::utility::TypeNametoTypeID;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static TypeA: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"A\0") };

pub static TypeAAAA: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"AAAA\0") };

pub static TypeBoth: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"A+AAAA\0") };

pub static TypeAny: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"ANY\0") };

pub static TypeCNAME: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"CNAME\0") };

pub static TypeMX: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"MX\0") };

pub static TypeNS: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"NS\0") };

pub static TypePTR: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"PTR\0") };

pub static TypeSOA: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"SOA\0") };

pub static TypeSRV: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"SRV\0") };

pub static ClassIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"IN\0") };

pub static ClassAny: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"ANY\0") };

pub const DEFAULT_ROOT: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"A.ROOT-SERVERS.NET.\0")
};

pub static ARPA_SIG: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b".in-addr.arpa\0") };

#[no_mangle]
pub static mut State: STATE = _STATE {
    debug: 0,
    defname: 0,
    d2: 0,
    recurse: 0,
    search: 0,
    vc: 0,
    ignoretc: 0,
    MSxfr: 0,
    domain: [0; 256],
    srchlist: [[0; 256]; 6],
    root: [0; 256],
    retry: 0,
    timeout: 0,
    ixfrver: 0,
    type_: ::core::ptr::null::<CHAR>() as *mut CHAR,
    Class: ::core::ptr::null::<CHAR>() as *mut CHAR,
    port: 0,
    DefaultServer: [0; 256],
    DefaultServerAddress: [0; 16],
};

#[no_mangle]
pub static mut ProcessHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut RequestID: ULONG = 0;
#[no_mangle]
pub unsafe extern "C" fn PrintState() {
    _tprintf( b"Default Server:  %s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(State.DefaultServer) as *mut CHAR );
    _tprintf( b"Address:  %s\n\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(State.DefaultServerAddress) as *mut CHAR );

    _tprintf( b"Set options:\n\0".as_ptr() as *const ::core::ffi::c_char );

    _tprintf( b"  \0".as_ptr() as *const ::core::ffi::c_char );
    if State.debug == 0 { _tprintf( b"no\0".as_ptr() as *const ::core::ffi::c_char ); }
    _tprintf( b"debug\n\0".as_ptr() as *const ::core::ffi::c_char );

    _tprintf( b"  \0".as_ptr() as *const ::core::ffi::c_char );
    if State.defname == 0 { _tprintf( b"no\0".as_ptr() as *const ::core::ffi::c_char ); }
    _tprintf( b"defname\n\0".as_ptr() as *const ::core::ffi::c_char );

    _tprintf( b"  \0".as_ptr() as *const ::core::ffi::c_char );
    if State.search == 0 { _tprintf( b"no\0".as_ptr() as *const ::core::ffi::c_char ); }
    _tprintf( b"search\n\0".as_ptr() as *const ::core::ffi::c_char );

    _tprintf( b"  \0".as_ptr() as *const ::core::ffi::c_char );
    if State.recurse == 0 { _tprintf( b"no\0".as_ptr() as *const ::core::ffi::c_char ); }
    _tprintf( b"recurse\n\0".as_ptr() as *const ::core::ffi::c_char );

    _tprintf( b"  \0".as_ptr() as *const ::core::ffi::c_char );
    if State.d2 == 0 { _tprintf( b"no\0".as_ptr() as *const ::core::ffi::c_char ); }
    _tprintf( b"d2\n\0".as_ptr() as *const ::core::ffi::c_char );

    _tprintf( b"  \0".as_ptr() as *const ::core::ffi::c_char );
    if State.vc == 0 { _tprintf( b"no\0".as_ptr() as *const ::core::ffi::c_char ); }
    _tprintf( b"vc\n\0".as_ptr() as *const ::core::ffi::c_char );

    _tprintf( b"  \0".as_ptr() as *const ::core::ffi::c_char );
    if State.ignoretc == 0 { _tprintf( b"no\0".as_ptr() as *const ::core::ffi::c_char ); }
    _tprintf( b"ignoretc\n\0".as_ptr() as *const ::core::ffi::c_char );

    _tprintf( b"  port=%d\n\0".as_ptr() as *const ::core::ffi::c_char, State.port as i32 );
    _tprintf( b"  type=%s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(State.type_) as *mut CHAR );
    _tprintf( b"  class=%s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(State.Class) as *mut CHAR );
    _tprintf( b"  timeout=%d\n\0".as_ptr() as *const ::core::ffi::c_char, State.timeout as i32 );
    _tprintf( b"  retry=%d\n\0".as_ptr() as *const ::core::ffi::c_char, State.retry as i32 );
    _tprintf( b"  root=%s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(State.root) as *mut CHAR );
    _tprintf( b"  domain=%s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(State.domain) as *mut CHAR );

    _tprintf( b"  \0".as_ptr() as *const ::core::ffi::c_char );
    if State.MSxfr == 0 { _tprintf( b"no\0".as_ptr() as *const ::core::ffi::c_char ); }
    _tprintf( b"MSxfr\n\0".as_ptr() as *const ::core::ffi::c_char );

    _tprintf( b"  IXFRversion=%d\n\0".as_ptr() as *const ::core::ffi::c_char, State.ixfrver as i32 );

    _tprintf( b"  srchlist=%s\n\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(State.srchlist[0]) as *mut CHAR );
}
#[no_mangle]
pub unsafe extern "C" fn PrintUsage() {
    _tprintf(
        _T!(b"Usage:\n   nslookup [-opt ...]             # interactive mode using default server\n   nslookup [-opt ...] - server    # interactive mode using 'server'\n   nslookup [-opt ...] host        # just look up 'host' using default server\n   nslookup [-opt ...] host server # just look up 'host' using 'server'\n\0"
            .as_ptr() as *const i8)
    );
}
#[no_mangle]
pub unsafe extern "C" fn PrintHelp() {
    _tprintf(b"Commands:   (identifiers are shown in uppercase, [] means optional)\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"NAME1           - print info about host/domain NAME using default server\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"NAME1 NAME2     - as above, but use NAME2 as server\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"help or ?       - print info on common commands\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"set OPTION      - set an option\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    all                 - print options, current server and host\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    [no]debug           - print debugging information\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    [no]d2              - print exhaustive debugging information\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    [no]defname         - apend domain name to each query\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    [no]recurse         - ask for recursive answer to query\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    [no]search          - use domain list query\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    [no]vc              - always use virtual circuit\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    domain=NAME         - set default domain name to NAME\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    srchlist=N1[/N2/.../N6] - set domain to N1 and search list to N1,N2 etc.\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    root=NAME           - set root server to NAME\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    retry=X             - set number of retries to X\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    timeout=X           - set initial time-out interval to X seconds\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    type=X              - set query type (ex. A,ANY,CNAME,MX,NS,PTR,SOA,SRV)\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    querytype=X         - same as type\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    class=X             - set query type (ex. IN (Internet), ANY)\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    [no]msxfr           - use MS fast zone transfer\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"    ixfrver=X           - current version to use in IXFR transfer request\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"server NAME     - set default server to NAME, using current default server\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"lserver NAME    - set default server to NAME, using initial server\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"finger [USER]   - finger the optional NAME at the current default host\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"root            - set current default server too the root\n\0".as_ptr() as *const ::core::ffi::c_char);

    _tprintf(b"exit            - exit the program\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn PerformInternalLookup(pAddr: PCHAR, pResult: PCHAR) -> BOOL {
    /* Needed to issue DNS packets and parse them. */
    let mut Buffer: PCHAR = ::core::ptr::null_mut::<CHAR>();
    let mut RecBuffer: PCHAR = ::core::ptr::null_mut::<CHAR>();
    let mut pResolve: [CHAR; 256] = [0; 256];
    let mut BufferLength: ULONG = 0 as ULONG;
    let mut RecBufferLength: ULONG = 512 as ULONG;
    let mut i: i32 = 0 as i32;
    let mut j: i32 = 0 as i32;
    let mut k: i32 = 0 as i32;
    let mut d: i32 = 0 as i32;
    let mut bOk: BOOL = FALSE as BOOL;

    /* Makes things easier when parsing the response packet. */
    let mut NumQuestions: USHORT = 0;
    let mut Type: USHORT = 0;

    if (strlen(pAddr) as usize).wrapping_add(1 as usize) > 255 as usize {
        return (FALSE) as BOOL;
    }

    Type = TYPE_A as USHORT;
    if IsValidIP(pAddr) != 0 {
        Type = TYPE_PTR as USHORT;
    }

    /* If it's a PTR lookup then append the ARPA sig to the end. */
    if Type as i32 == TYPE_PTR as i32 {
        ReverseIP(pAddr, core::ptr::addr_of_mut!(pResolve) as PCHAR);
        strcat(core::ptr::addr_of_mut!(pResolve) as PCHAR, (ARPA_SIG).as_ptr());
    } else {
        strcpy(core::ptr::addr_of_mut!(pResolve) as PCHAR, pAddr);
    }

    /* Base header length + length of QNAME + length of QTYPE and QCLASS */
    BufferLength = (12 as usize)
        .wrapping_add(
            (strlen(core::ptr::addr_of_mut!(pResolve) as PCHAR) as usize)
                .wrapping_add(2 as usize),
        )
        .wrapping_add(4 as usize) as ULONG;

    /* Allocate memory for the buffer. */
    Buffer = HeapAlloc(ProcessHeap, 0 as DWORD, BufferLength as SIZE_T) as PCHAR;
    'cleanup: {
        if Buffer.is_null() {
            _tprintf(b"ERROR: Out of memory\n\0".as_ptr() as *const ::core::ffi::c_char);
            break 'cleanup;
        }

        /* Allocate the receiving buffer. */
        RecBuffer = HeapAlloc(ProcessHeap, 0 as DWORD, RecBufferLength as SIZE_T) as PCHAR;
        if RecBuffer.is_null() {
            _tprintf(b"ERROR: Out of memory\n\0".as_ptr() as *const ::core::ffi::c_char);
            break 'cleanup;
        }

        /* Insert the ID field. */
        *(Buffer.offset(i as isize) as *mut CHAR as PSHORT).offset(0 as isize) =
            htons(RequestID as u_short) as SHORT;
        i += 2 as i32;

        /* Bits 0-7 of the second 16 are all 0, except for when recursion is
           desired. */
        *Buffer.offset(i as isize) = 0x00 as CHAR;
        if State.recurse != 0 {
            *Buffer.offset(i as isize) |= 0x01 as CHAR;
        }
        i += 1 as i32;

        /* Bits 8-15 of the second 16 are 0 for a query. */
        *Buffer.offset(i as isize) = 0x00 as CHAR;
        i += 1 as i32;

        /* Only 1 question. */
        *(Buffer.offset(i as isize) as *mut CHAR as PSHORT).offset(0 as isize) =
            htons(1 as u_short) as SHORT;
        i += 2 as i32;

        /* We aren't sending a response, so 0 out the rest of the header. */
        *Buffer.offset(i as isize) = 0x00 as CHAR;
        *Buffer.offset((i + 1 as i32) as isize) = 0x00 as CHAR;
        *Buffer.offset((i + 2 as i32) as isize) = 0x00 as CHAR;
        *Buffer.offset((i + 3 as i32) as isize) = 0x00 as CHAR;
        *Buffer.offset((i + 4 as i32) as isize) = 0x00 as CHAR;
        *Buffer.offset((i + 5 as i32) as isize) = 0x00 as CHAR;
        i += 6 as i32;

        /* Walk through the query address. Split each section delimited by '.'.
           Format of the QNAME section is length|data, etc. Last one is null */
        j = i;
        i += 1 as i32;

        k = 0 as i32;
        while (k as usize) < strlen(core::ptr::addr_of_mut!(pResolve) as PCHAR) {
            if pResolve[k as usize] as i32 != '.' as i32 {
                *Buffer.offset(i as isize) = pResolve[k as usize];
                i += 1 as i32;
            } else {
                *Buffer.offset(j as isize) = ((i - j) - 1 as i32) as CHAR;
                j = i;
                i += 1 as i32;
            }
            k += 1 as i32;
        }

        *Buffer.offset(j as isize) = ((i - j) - 1 as i32) as CHAR;
        *Buffer.offset(i as isize) = 0x00 as CHAR;
        i += 1 as i32;

        /* QTYPE */
        *(Buffer.offset(i as isize) as *mut CHAR as PSHORT).offset(0 as isize) =
            htons(Type as u_short) as SHORT;
        i += 2 as i32;

        /* QCLASS */
        *(Buffer.offset(i as isize) as *mut CHAR as PSHORT).offset(0 as isize) =
            htons(CLASS_IN as u_short) as SHORT;

        /* Ship the request off to the DNS server. */
        bOk = SendRequest(
            Buffer,
            BufferLength,
            RecBuffer,
            core::ptr::addr_of_mut!(RecBufferLength),
        );
        if bOk == 0 {
            break 'cleanup;
        }

        /* Start parsing the received packet. */
        NumQuestions = ntohs(
            *(RecBuffer.offset(4 as isize) as *mut CHAR as PSHORT).offset(0 as isize) as u_short,
        ) as USHORT;

        k = 12 as i32;

        /* We don't care about the questions section, blow through it. */
        if NumQuestions != 0 {
            i = 0 as i32;
            while i < NumQuestions as i32 {
                /* Quick way to skip the domain name section. */
                k += ExtractName(RecBuffer, pResult, k as USHORT, 0 as UCHAR) as i32;
                k += 4 as i32;
                i += 1 as i32;
            }
        }

        /* Skip the answer name. */
        k += ExtractName(RecBuffer, pResult, k as USHORT, 0 as UCHAR) as i32;

        Type = ntohs(
            *(RecBuffer.offset(k as isize) as *mut CHAR as PUSHORT).offset(0 as isize) as u_short,
        ) as USHORT;
        k += 8 as i32;

        d = ntohs(
            *(RecBuffer.offset(k as isize) as *mut CHAR as PUSHORT).offset(0 as isize) as u_short,
        ) as i32;
        k += 2 as i32;

        if TYPE_PTR as i32 == Type as i32 {
            k += ExtractName(RecBuffer, pResult, k as USHORT, d as UCHAR) as i32;
        } else if TYPE_A as i32 == Type as i32 {
            k += ExtractIP(RecBuffer, pResult, k as USHORT) as i32;
        }
    }

    /* Free memory. */
    if !Buffer.is_null() {
        HeapFree(ProcessHeap, 0 as DWORD, Buffer as PVOID);
    }
    if !RecBuffer.is_null() {
        HeapFree(ProcessHeap, 0 as DWORD, RecBuffer as PVOID);
    }

    RequestID = RequestID.wrapping_add(1 as ULONG);

    return bOk;
}
#[no_mangle]
pub unsafe extern "C" fn PerformLookup(mut pAddr: PCHAR) {
    /* Needed to issue DNS packets and parse them. */
    let mut Buffer: PCHAR = ::core::ptr::null_mut::<CHAR>();
    let mut RecBuffer: PCHAR = ::core::ptr::null_mut::<CHAR>();
    let mut pResolve: [CHAR; 256] = [0; 256];
    let mut pResult: [CHAR; 256] = [0; 256];
    let mut BufferLength: ULONG = 0 as ULONG;
    let mut RecBufferLength: ULONG = 512 as ULONG;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut d: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut bOk: BOOL = FALSE as BOOL;

    /* Makes things easier when parsing the response packet. */
    let mut Header2: UCHAR = 0;
    let mut NumQuestions: USHORT = 0;
    let mut NumAnswers: USHORT = 0;
    let mut NumAuthority: USHORT = 0;
    let mut Type: USHORT = 0;

    if (strlen( pAddr as *const ::core::ffi::c_char ) as size_t
        + 1 as size_t) > 255 as size_t
    {
        return;
    }

    _tprintf(
        _T!(b"Server:  %s\n\0".as_ptr() as *const ::core::ffi::c_char),
        &raw mut State.DefaultServer as *mut CHAR,
    );
    _tprintf(
        _T!(b"Address:  %s\n\n\0".as_ptr() as *const ::core::ffi::c_char),
        &raw mut State.DefaultServerAddress as *mut CHAR,
    );

    if strcmp(
        (TypeA).as_ptr() as *const ::core::ffi::c_char,
        State.type_ as *const ::core::ffi::c_char,
    ) == 0
        || strcmp(
            TypeAAAA.as_ptr(),
            State.type_ as *const ::core::ffi::c_char,
        ) == 0
        || strcmp(
            TypeBoth.as_ptr(),
            State.type_ as *const ::core::ffi::c_char,
        ) == 0
    {
        Type = TYPE_A as USHORT;
        if IsValidIP( pAddr ) != 0 {
            Type = TYPE_PTR as USHORT;
        }
    } else {
        Type = TypeNametoTypeID( State.type_ ) as USHORT;
    }

    /* If it's a PTR lookup then append the ARPA sig to the end. */
    if (Type as ::core::ffi::c_int == TYPE_PTR as ::core::ffi::c_int)
        && IsValidIP( pAddr ) != 0
    {
        ReverseIP( pAddr, &raw mut pResolve as PCHAR );
        strcat(
            &raw mut pResolve as *mut ::core::ffi::c_char,
            (ARPA_SIG).as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        strcpy(
            &raw mut pResolve as *mut ::core::ffi::c_char,
            pAddr as *const ::core::ffi::c_char,
        );
    }

    /* Base header length + length of QNAME + length of QTYPE and QCLASS */
    BufferLength = (12 as size_t
        + (strlen( &raw mut pResolve as *const ::core::ffi::c_char )
            + 2 as size_t)
        + 4 as size_t) as ULONG;

    'cleanup: {
        /* Allocate memory for the buffer. */
        Buffer = HeapAlloc(
            ProcessHeap,
            0 as DWORD,
            BufferLength as SIZE_T,
        ) as PCHAR;
        if Buffer.is_null() {
            _tprintf( _T!(b"ERROR: Out of memory\n\0".as_ptr() as *const ::core::ffi::c_char) );
            break 'cleanup;
        }

        /* Allocate memory for the return buffer. */
        RecBuffer = HeapAlloc(
            ProcessHeap,
            0 as DWORD,
            RecBufferLength as SIZE_T,
        ) as PCHAR;
        if RecBuffer.is_null() {
            _tprintf( _T!(b"ERROR: Out of memory\n\0".as_ptr() as *const ::core::ffi::c_char) );
            break 'cleanup;
        }

        /* Insert the ID field. */
        *(Buffer.offset(i as isize) as PSHORT).offset(0 as isize) =
            htons( RequestID as USHORT ) as SHORT;
        i += 2 as ::core::ffi::c_int;

        /* Bits 0-7 of the second 16 are all 0, except for when recursion is
        desired. */
        *Buffer.offset(i as isize) = 0x00 as CHAR;
        if State.recurse != 0 {
            *Buffer.offset(i as isize) |= 0x01 as CHAR;
        }
        i += 1 as ::core::ffi::c_int;

        /* Bits 8-15 of the second 16 are 0 for a query. */
        *Buffer.offset(i as isize) = 0x00 as CHAR;
        i += 1 as ::core::ffi::c_int;

        /* Only 1 question. */
        *(Buffer.offset(i as isize) as PSHORT).offset(0 as isize) =
            htons( 1 as USHORT ) as SHORT;
        i += 2 as ::core::ffi::c_int;

        /* We aren't sending a response, so 0 out the rest of the header. */
        *Buffer.offset(i as isize) = 0x00 as CHAR;
        *Buffer.offset((i + 1 as ::core::ffi::c_int) as isize) =
            0x00 as CHAR;
        *Buffer.offset((i + 2 as ::core::ffi::c_int) as isize) =
            0x00 as CHAR;
        *Buffer.offset((i + 3 as ::core::ffi::c_int) as isize) =
            0x00 as CHAR;
        *Buffer.offset((i + 4 as ::core::ffi::c_int) as isize) =
            0x00 as CHAR;
        *Buffer.offset((i + 5 as ::core::ffi::c_int) as isize) =
            0x00 as CHAR;
        i += 6 as ::core::ffi::c_int;

        /* Walk through the query address. Split each section delimited by '.'.
           Format of the QNAME section is length|data, etc. Last one is null */
        j = i;
        i += 1 as ::core::ffi::c_int;

        k = 0 as ::core::ffi::c_int;
        while (k as size_t) < strlen( &raw mut pResolve as *const ::core::ffi::c_char ) {
            if pResolve[k as usize] as ::core::ffi::c_int != '.' as ::core::ffi::c_int {
                *Buffer.offset(i as isize) = pResolve[k as usize];
                i += 1 as ::core::ffi::c_int;
            } else {
                *Buffer.offset(j as isize) =
                    (i - j - 1 as ::core::ffi::c_int) as CHAR;
                j = i;
                i += 1 as ::core::ffi::c_int;
            }
            k += 1 as ::core::ffi::c_int;
        }

        *Buffer.offset(j as isize) = (i - j - 1 as ::core::ffi::c_int) as CHAR;
        *Buffer.offset(i as isize) = 0x00 as CHAR;
        i += 1 as ::core::ffi::c_int;

        /* QTYPE */
        *(Buffer.offset(i as isize) as PSHORT).offset(0 as isize) =
            htons( Type as USHORT ) as SHORT;
        i += 2 as ::core::ffi::c_int;

        /* QCLASS */
        *(Buffer.offset(i as isize) as PSHORT).offset(0 as isize) =
            htons( ClassNametoClassID( State.Class ) as USHORT ) as SHORT;

        /* Ship off the request to the DNS server. */
        bOk = SendRequest(
            Buffer,
            BufferLength,
            RecBuffer,
            &raw mut RecBufferLength,
        );
        if bOk == 0 {
            break 'cleanup;
        }

        /* Start parsing the received packet. */
        Header2 = (*RecBuffer.offset(3 as isize)) as u8;
        NumQuestions = ntohs(
            (*(RecBuffer.offset(4 as isize) as PSHORT)
                .offset(0 as isize)) as u16,
        ) as USHORT;
        NumAnswers = ntohs(
            (*(RecBuffer.offset(6 as isize) as PSHORT)
                .offset(0 as isize)) as u16,
        ) as USHORT;
        NumAuthority = ntohs(
            *(RecBuffer.offset(8 as isize) as PUSHORT)
                .offset(0 as isize),
        ) as USHORT;
        Type = 0 as USHORT;

        /* Check the RCODE for failure. */
        d = Header2 as ::core::ffi::c_int & 0x0F as ::core::ffi::c_int;
        if d != RCODE_NOERROR as ::core::ffi::c_int {
            match (d) as u32 {
                RCODE_NXDOMAIN => {
                    _tprintf(
                        _T!(b"*** %s can't find %s: Non-existant domain\n\0".as_ptr() as *const ::core::ffi::c_char),
                        &raw mut State.DefaultServer as *mut CHAR,
                        pAddr,
                    );
                }
                RCODE_REFUSED => {
                    _tprintf(
                        _T!(b"*** %s can't find %s: Query refused\n\0".as_ptr() as *const ::core::ffi::c_char),
                        &raw mut State.DefaultServer as *mut CHAR,
                        pAddr,
                    );
                }
                _ => {
                    _tprintf(
                        _T!(b"*** %s can't find %s: Unknown RCODE\n\0".as_ptr() as *const ::core::ffi::c_char),
                        &raw mut State.DefaultServer as *mut CHAR,
                        pAddr,
                    );
                }
            }

            break 'cleanup;
        }

        k = 12 as ::core::ffi::c_int;

        if NumQuestions != 0 {
            /* Blow through the questions section since we don't care about it. */
            i = 0 as ::core::ffi::c_int;
            while i < NumQuestions as ::core::ffi::c_int {
                k += ExtractName(
                    RecBuffer,
                    &raw mut pResult as PCHAR,
                    k as USHORT,
                    0 as UCHAR,
                );
                k += 4 as ::core::ffi::c_int;
                i += 1 as ::core::ffi::c_int;
            }
        }

        if NumAnswers != 0 {
            /* Skip the name. */
            k += ExtractName(
                RecBuffer,
                &raw mut pResult as PCHAR,
                k as USHORT,
                0 as UCHAR,
            );

            Type = ntohs(
                *(RecBuffer.offset(k as isize) as PUSHORT)
                    .offset(0 as isize),
            ) as USHORT;
            k += 8 as ::core::ffi::c_int;

            d = ntohs(
                *(RecBuffer.offset(k as isize) as PUSHORT)
                    .offset(0 as isize),
            ) as ::core::ffi::c_int;
            k += 2 as ::core::ffi::c_int;

            if TYPE_PTR as ::core::ffi::c_int == Type as ::core::ffi::c_int {
                k += ExtractName(
                    RecBuffer,
                    &raw mut pResult as PCHAR,
                    k as USHORT,
                    d as UCHAR,
                );
            } else if TYPE_A as ::core::ffi::c_int == Type as ::core::ffi::c_int {
                k += ExtractIP(
                    RecBuffer,
                    &raw mut pResult as PCHAR,
                    k as USHORT,
                );
            }
        }

        /* FIXME: This'll need to support more than PTR and A at some point. */
        if strcmp(
            State.type_ as *const ::core::ffi::c_char,
            TypePTR.as_ptr(),
        ) == 0
        {
            if TYPE_PTR as ::core::ffi::c_int == Type as ::core::ffi::c_int {
                _tprintf(
                    _T!(b"%s     name = %s\n\0".as_ptr() as *const ::core::ffi::c_char),
                    &raw mut pResolve as *mut CHAR,
                    &raw mut pResult as *mut CHAR,
                );
            } else {
            }
        } else if strcmp(
            State.type_ as *const ::core::ffi::c_char,
            (TypeA).as_ptr() as *const ::core::ffi::c_char,
        ) == 0
            || strcmp(
                State.type_ as *const ::core::ffi::c_char,
                TypeAAAA.as_ptr(),
            ) == 0
            || strcmp(
                State.type_ as *const ::core::ffi::c_char,
                TypeBoth.as_ptr(),
            ) == 0
        {
            if TYPE_A as ::core::ffi::c_int == Type as ::core::ffi::c_int {
                if 0 as ::core::ffi::c_int == NumAuthority as ::core::ffi::c_int {
                    _tprintf(
                        _T!(b"Non-authoritative answer:\n\0".as_ptr() as *const ::core::ffi::c_char),
                    );
                }

                _tprintf(
                    _T!(b"Name:    %s\n\0".as_ptr() as *const ::core::ffi::c_char),
                    pAddr,
                );
                _tprintf(
                    _T!(b"Address:  %s\n\n\0".as_ptr() as *const ::core::ffi::c_char),
                    &raw mut pResult as PCHAR,
                );
            } else {
                _tprintf(
                    _T!(b"Name:    %s\n\0".as_ptr() as *const ::core::ffi::c_char),
                    &raw mut pResult as PCHAR,
                );
                _tprintf(
                    _T!(b"Address:  %s\n\n\0".as_ptr() as *const ::core::ffi::c_char),
                    pAddr,
                );
            }
        }
    }

    /* Free memory. */
    if !Buffer.is_null() {
        HeapFree( ProcessHeap, 0 as DWORD, Buffer as PVOID );
    }
    if !RecBuffer.is_null() {
        HeapFree( ProcessHeap, 0 as DWORD, RecBuffer as PVOID );
    }

    RequestID += 1 as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn ParseCommandLine(argc: core::ffi::c_int, argv: *mut *mut core::ffi::c_char) -> BOOL {
    let mut i: core::ffi::c_int = 0;
    let mut NoMoreOptions: BOOL = (FALSE as i32);
    let mut Interactive: BOOL = (FALSE as i32);
    let mut AddrToResolve: [CHAR; 256] = [0; 256];
    let mut Server: [CHAR; 256] = [0; 256];
    let mut dwValue: DWORD = 0;
    let mut pszEnd: PSTR = core::ptr::null_mut();
    let mut pszValue: PSTR = core::ptr::null_mut();

    RtlZeroMemory!(core::ptr::addr_of_mut!(AddrToResolve) as *mut core::ffi::c_void, 256 as usize);
    RtlZeroMemory!(core::ptr::addr_of_mut!(Server) as *mut core::ffi::c_void, 256 as usize);

    if 2 == argc {
        /* In the Windows nslookup, usage is only displayed if /? is the only
           option specified on the command line. */
        if strncmp(b"/?\0".as_ptr() as *const core::ffi::c_char, *argv.offset(1 as isize), 2 as usize) == 0 {
            PrintUsage();
            return 0;
        }
    }

    if argc > 1 {
        i = 1;
        while i < argc {
            if NoMoreOptions != 0 {
                strncpy((core::ptr::addr_of_mut!(Server) as *mut core::ffi::c_char).cast(), *argv.offset(i as isize) as *const u8, 255 as usize);

                /* Determine which one to resolve. This is based on whether the
                   DNS server provided was an IP or an FQDN. */
                if IsValidIP(core::ptr::addr_of_mut!(Server) as PCHAR) != 0 {
                    strncpy((core::ptr::addr_of_mut!(State.DefaultServerAddress) as *mut core::ffi::c_char).cast(), core::ptr::addr_of_mut!(Server) as *const u8, 16 as usize);

                    PerformInternalLookup(core::ptr::addr_of_mut!(State.DefaultServerAddress) as PCHAR,
                                           core::ptr::addr_of_mut!(State.DefaultServer) as PCHAR);
                } else {
                    strncpy((core::ptr::addr_of_mut!(State.DefaultServer) as *mut core::ffi::c_char).cast(), core::ptr::addr_of_mut!(Server) as *const u8, 255 as usize);

                    PerformInternalLookup(core::ptr::addr_of_mut!(State.DefaultServer) as PCHAR,
                                           core::ptr::addr_of_mut!(State.DefaultServerAddress) as PCHAR);
                }

                if Interactive != 0 {
                    return 1;
                }

                PerformLookup(core::ptr::addr_of_mut!(AddrToResolve) as PCHAR);

                return 0;
            } else {
                if strncmp(b"-all\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 4 as usize) == 0 {
                    PrintState();
                } else if strncmp(b"-type=\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 6 as usize) == 0 {
                    if strncmp((TypeA).as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen((TypeA).as_ptr())) == 0 {
                        State.type_ = TypeA.as_ptr() as PCHAR;
                    } else if strncmp(TypeAAAA.as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen(TypeAAAA.as_ptr())) == 0 {
                        State.type_ = TypeAAAA.as_ptr() as PCHAR;
                    } else if strncmp(TypeBoth.as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen(TypeBoth.as_ptr())) == 0 {
                        State.type_ = TypeBoth.as_ptr() as PCHAR;
                    } else if strncmp(TypeAny.as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen(TypeAny.as_ptr())) == 0 {
                        State.type_ = TypeAny.as_ptr() as PCHAR;
                    } else if strncmp(TypeCNAME.as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen(TypeCNAME.as_ptr())) == 0 {
                        State.type_ = TypeCNAME.as_ptr() as PCHAR;
                    } else if strncmp(TypeMX.as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen(TypeMX.as_ptr())) == 0 {
                        State.type_ = TypeMX.as_ptr() as PCHAR;
                    } else if strncmp(TypeNS.as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen(TypeNS.as_ptr())) == 0 {
                        State.type_ = TypeNS.as_ptr() as PCHAR;
                    } else if strncmp(TypePTR.as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen(TypePTR.as_ptr())) == 0 {
                        State.type_ = TypePTR.as_ptr() as PCHAR;
                    } else if strncmp(TypeSOA.as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen(TypeSOA.as_ptr())) == 0 {
                        State.type_ = TypeSOA.as_ptr() as PCHAR;
                    } else if strncmp(TypeSRV.as_ptr(), (*argv.offset(i as isize)).offset(6 as isize), strlen(TypeSRV.as_ptr())) == 0 {
                        State.type_ = TypeSRV.as_ptr() as PCHAR;
                    } else {
                        _tprintf(_T!(b"unknown query type: %s\0".as_ptr() as *const core::ffi::c_char), (*argv.offset(i as isize)).offset(6 as isize));
                    }
                } else if strncmp(b"-domain=\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 8 as usize) == 0 {
                    strcpy(core::ptr::addr_of_mut!(State.domain) as *mut core::ffi::c_char, (*argv.offset(i as isize)).offset(8 as isize));
                } else if strncmp(b"-srchlist=\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 10 as usize) == 0 {
                } else if strncmp(b"-root=\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 6 as usize) == 0 {
                    strcpy(core::ptr::addr_of_mut!(State.root) as *mut core::ffi::c_char, (*argv.offset(i as isize)).offset(6 as isize));
                } else if strncmp(b"-retry=\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 7 as usize) == 0 {
                    pszValue = (*argv.offset(i as isize)).offset(7 as isize) as PSTR;
                    dwValue = strtoul(pszValue as *const core::ffi::c_char, core::ptr::addr_of_mut!(pszEnd) as *mut *const core::ffi::c_char, 10) as DWORD;
                    if pszEnd != pszValue {
                        State.retry = dwValue;
                    }
                } else if strncmp(b"-timeout=\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 9 as usize) == 0 {
                    pszValue = (*argv.offset(i as isize)).offset(9 as isize) as PSTR;
                    dwValue = strtoul(pszValue as *const core::ffi::c_char, core::ptr::addr_of_mut!(pszEnd) as *mut *const core::ffi::c_char, 10) as DWORD;
                    if pszEnd != pszValue {
                        State.timeout = dwValue;
                    }
                } else if strncmp(b"-querytype=\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 11 as usize) == 0 {
                    if strncmp((TypeA).as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen((TypeA).as_ptr())) == 0 {
                        State.type_ = TypeA.as_ptr() as PCHAR;
                    } else if strncmp(TypeAAAA.as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen(TypeAAAA.as_ptr())) == 0 {
                        State.type_ = TypeAAAA.as_ptr() as PCHAR;
                    } else if strncmp(TypeBoth.as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen(TypeBoth.as_ptr())) == 0 {
                        State.type_ = TypeBoth.as_ptr() as PCHAR;
                    } else if strncmp(TypeAny.as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen(TypeAny.as_ptr())) == 0 {
                        State.type_ = TypeAny.as_ptr() as PCHAR;
                    } else if strncmp(TypeCNAME.as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen(TypeCNAME.as_ptr())) == 0 {
                        State.type_ = TypeCNAME.as_ptr() as PCHAR;
                    } else if strncmp(TypeMX.as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen(TypeMX.as_ptr())) == 0 {
                        State.type_ = TypeMX.as_ptr() as PCHAR;
                    } else if strncmp(TypeNS.as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen(TypeNS.as_ptr())) == 0 {
                        State.type_ = TypeNS.as_ptr() as PCHAR;
                    } else if strncmp(TypePTR.as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen(TypePTR.as_ptr())) == 0 {
                        State.type_ = TypePTR.as_ptr() as PCHAR;
                    } else if strncmp(TypeSOA.as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen(TypeSOA.as_ptr())) == 0 {
                        State.type_ = TypeSOA.as_ptr() as PCHAR;
                    } else if strncmp(TypeSRV.as_ptr(), (*argv.offset(i as isize)).offset(11 as isize), strlen(TypeSRV.as_ptr())) == 0 {
                        State.type_ = TypeSRV.as_ptr() as PCHAR;
                    } else {
                        _tprintf(_T!(b"unknown query type: %s\0".as_ptr() as *const core::ffi::c_char), (*argv.offset(i as isize)).offset(11 as isize));
                    }
                } else if strncmp(b"-class=\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 7 as usize) == 0 {
                    if strncmp(ClassIN.as_ptr(), (*argv.offset(i as isize)).offset(7 as isize), strlen(ClassIN.as_ptr())) == 0 {
                        State.Class = ClassIN.as_ptr() as PCHAR;
                    } else if strncmp(ClassAny.as_ptr(), (*argv.offset(i as isize)).offset(7 as isize), strlen(ClassAny.as_ptr())) == 0 {
                        State.Class = ClassAny.as_ptr() as PCHAR;
                    } else {
                        _tprintf(_T!(b"unknown query class: %s\0".as_ptr() as *const core::ffi::c_char), (*argv.offset(i as isize)).offset(7 as isize));
                    }
                } else if strncmp(b"-ixfrver=\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 9 as usize) == 0 {
                } else if strncmp(b"-debug\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 6 as usize) == 0 {
                    State.debug = (TRUE as i32);
                } else if strncmp(b"-nodebug\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 8 as usize) == 0 {
                    State.debug = (FALSE as i32);
                    State.d2 = (FALSE as i32);
                } else if strncmp(b"-d2\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 3 as usize) == 0 {
                    State.d2 = (TRUE as i32);
                    State.debug = (TRUE as i32);
                } else if strncmp(b"-nod2\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 5 as usize) == 0 {
                    if State.debug != 0 {
                        _tprintf(_T!(b"d2 mode disabled; still in debug mode\n\0".as_ptr() as *const core::ffi::c_char));
                    }

                    State.d2 = (FALSE as i32);
                } else if strncmp(b"-defname\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 8 as usize) == 0 {
                    State.defname = (TRUE as i32);
                } else if strncmp(b"-noddefname\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 10 as usize) == 0 {
                    State.defname = (FALSE as i32);
                } else if strncmp(b"-recurse\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 8 as usize) == 0 {
                    State.recurse = (TRUE as i32);
                } else if strncmp(b"-norecurse\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 10 as usize) == 0 {
                    State.recurse = (FALSE as i32);
                } else if strncmp(b"-search\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 7 as usize) == 0 {
                    State.search = (TRUE as i32);
                } else if strncmp(b"-nosearch\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 9 as usize) == 0 {
                    State.search = (FALSE as i32);
                } else if strncmp(b"-vc\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 3 as usize) == 0 {
                    State.vc = (TRUE as i32);
                } else if strncmp(b"-novc\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 5 as usize) == 0 {
                    State.vc = (FALSE as i32);
                } else if strncmp(b"-msxfr\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 6 as usize) == 0 {
                    State.MSxfr = (TRUE as i32);
                } else if strncmp(b"-nomsxfr\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 8 as usize) == 0 {
                    State.MSxfr = (FALSE as i32);
                } else if strncmp(b"-\0".as_ptr() as *const core::ffi::c_char, *argv.offset(i as isize), 1 as usize) == 0 && (strlen(*argv.offset(i as isize)) == 1 as usize) {
                    /* Since we received just the plain - switch, we are going
                       to be entering interactive mode. We also will not be
                       parsing any more options. */
                    NoMoreOptions = (TRUE as i32);
                    Interactive = (TRUE as i32);
                } else {
                    /* Grab the address to resolve. No more options accepted
                       past this point. */
                    strncpy((core::ptr::addr_of_mut!(AddrToResolve) as *mut core::ffi::c_char).cast(), *argv.offset(i as isize) as *const u8, 255 as usize);
                    NoMoreOptions = (TRUE as i32);
                }
            }
            i += 1;
        }

        if NoMoreOptions != 0 && Interactive == 0 {
            /* Get the FQDN of the DNS server. */
            PerformInternalLookup(core::ptr::addr_of_mut!(State.DefaultServerAddress) as PCHAR,
                                   core::ptr::addr_of_mut!(State.DefaultServer) as PCHAR);

            PerformLookup(core::ptr::addr_of_mut!(AddrToResolve) as PCHAR);

            return 0;
        }
    }

    /* Get the FQDN of the DNS server. */
    PerformInternalLookup(core::ptr::addr_of_mut!(State.DefaultServerAddress) as PCHAR,
                           core::ptr::addr_of_mut!(State.DefaultServer) as PCHAR);

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn InteractiveMode() {
    let mut input_line: [CHAR; 256] = [0; 256];
    let mut args_vector: [PSTR; 64] = [core::ptr::null_mut(); 64];
    let mut dwArgCount: DWORD = 0 as DWORD;
    let mut bWhiteSpace: BOOL = TRUE as BOOL;
    let mut bDone: BOOL = FALSE as BOOL;
    let mut bInQuotes: BOOL = FALSE as BOOL;
    let mut ptr: PSTR = core::ptr::null_mut();
    let mut dwValue: DWORD = 0;
    let mut pszEnd: PSTR = core::ptr::null_mut();
    let mut pszValue: PSTR = core::ptr::null_mut();

    _tprintf(_T!(b"Default Server:  %s\n\0".as_ptr() as *const CHAR), core::ptr::addr_of_mut!(State.DefaultServer) as *mut CHAR);
    _tprintf(_T!(b"Address:  %s\n\n\0".as_ptr() as *const CHAR), core::ptr::addr_of_mut!(State.DefaultServerAddress) as *mut CHAR);

    loop {
        dwArgCount = 0 as DWORD;
        memset(core::ptr::addr_of_mut!(args_vector) as *mut ::core::ffi::c_void, 0 as i32, core::mem::size_of::<[PSTR; 64]>());

        _tprintf(_T!(b"> \0".as_ptr() as *const CHAR));

        /* Get input from the user. */
        fgets(core::ptr::addr_of_mut!(input_line) as *mut CHAR, 256 as i32, __iob_func().add(0));

        ptr = core::ptr::addr_of_mut!(input_line) as *mut CHAR as PSTR;
        while *ptr as i32 != 0 as i32 {
            if *ptr == _T!('"' as CHAR) {
                bInQuotes = (if bInQuotes != 0 { FALSE } else { TRUE }) as BOOL;
            }

            if (_istspace(*ptr as i32) != 0 && (bInQuotes == FALSE as BOOL)) || *ptr == _T!('\n' as CHAR)
            {
                *ptr = _T!('\0' as CHAR);
                bWhiteSpace = TRUE as BOOL;
            }
            else
            {
                if (bWhiteSpace != FALSE as BOOL) && (dwArgCount < 64 as DWORD)
                {
                    args_vector[dwArgCount as usize] = ptr;
                    dwArgCount = dwArgCount.wrapping_add(1);
                }
                bWhiteSpace = FALSE as BOOL;
            }
            ptr = ptr.offset(1);
        }

        if dwArgCount > 0 as DWORD
        {
            if _stricmp(args_vector[0 as usize] as *const CHAR, b"exit\0".as_ptr() as *const CHAR) == 0
            {
                bDone = TRUE as BOOL;
            }
            else if (_stricmp(args_vector[0 as usize] as *const CHAR, b"help\0".as_ptr() as *const CHAR) == 0) ||
                    (_stricmp(args_vector[0 as usize] as *const CHAR, b"?\0".as_ptr() as *const CHAR) == 0)
            {
                PrintHelp();
            }
            else if _stricmp(args_vector[0 as usize] as *const CHAR, b"set\0".as_ptr() as *const CHAR) == 0
            {
                if dwArgCount > 1 as DWORD
                {
                    if _stricmp(args_vector[1 as usize] as *const CHAR, b"all\0".as_ptr() as *const CHAR) == 0
                    {
                        PrintState();
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"debug\0".as_ptr() as *const CHAR) == 0
                    {
                        State.debug = TRUE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"nodebug\0".as_ptr() as *const CHAR) == 0
                    {
                        State.debug = FALSE as BOOL;
                        State.d2 = FALSE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"defname\0".as_ptr() as *const CHAR) == 0
                    {
                        State.defname = TRUE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"nodefname\0".as_ptr() as *const CHAR) == 0
                    {
                        State.defname = FALSE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"search\0".as_ptr() as *const CHAR) == 0
                    {
                        State.search = TRUE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"nosearch\0".as_ptr() as *const CHAR) == 0
                    {
                        State.search = FALSE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"recurse\0".as_ptr() as *const CHAR) == 0
                    {
                        State.recurse = TRUE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"norecurse\0".as_ptr() as *const CHAR) == 0
                    {
                        State.recurse = FALSE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"d2\0".as_ptr() as *const CHAR) == 0
                    {
                        State.debug = TRUE as BOOL;
                        State.d2 = TRUE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"nod2\0".as_ptr() as *const CHAR) == 0
                    {
                        State.d2 = FALSE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"vc\0".as_ptr() as *const CHAR) == 0
                    {
                        State.vc = TRUE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"novc\0".as_ptr() as *const CHAR) == 0
                    {
                        State.vc = FALSE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"msxfr\0".as_ptr() as *const CHAR) == 0
                    {
                        State.MSxfr = TRUE as BOOL;
                    }
                    else if _stricmp(args_vector[1 as usize] as *const CHAR, b"nomsxfr\0".as_ptr() as *const CHAR) == 0
                    {
                        State.MSxfr = FALSE as BOOL;
                    }
                    else if _strnicmp(args_vector[1 as usize] as *const CHAR, b"retry=\0".as_ptr() as *const CHAR, strlen(b"retry=\0".as_ptr() as *const CHAR)) == 0
                    {
                        pszValue = args_vector[1 as usize].add(strlen(b"retry=\0".as_ptr() as *const CHAR) as usize);
                        dwValue = strtoul(pszValue as *const CHAR, core::ptr::addr_of_mut!(pszEnd) as *mut PCSTR, 10 as i32) as DWORD;
                        if pszEnd != pszValue
                            { State.retry = dwValue; }
                    }
                    else if _strnicmp(args_vector[1 as usize] as *const CHAR, b"timeout=\0".as_ptr() as *const CHAR, strlen(b"timeout=\0".as_ptr() as *const CHAR)) == 0
                    {
                        pszValue = args_vector[1 as usize].add(strlen(b"timeout=\0".as_ptr() as *const CHAR) as usize);
                        dwValue = strtoul(pszValue as *const CHAR, core::ptr::addr_of_mut!(pszEnd) as *mut PCSTR, 10 as i32) as DWORD;
                        if pszEnd != pszValue
                            { State.timeout = dwValue; }
                    }
                    else if _strnicmp(args_vector[1 as usize] as *const CHAR, b"domain=\0".as_ptr() as *const CHAR, strlen(b"domain=\0".as_ptr() as *const CHAR)) == 0
                    {
                        strcpy(core::ptr::addr_of_mut!(State.domain) as *mut CHAR, args_vector[1 as usize].add(strlen(b"domain=\0".as_ptr() as *const CHAR) as usize));
                    }
                    else if _strnicmp(args_vector[1 as usize] as *const CHAR, b"srchlist=\0".as_ptr() as *const CHAR, strlen(b"srchlist=\0".as_ptr() as *const CHAR)) == 0
                    {
                        _tprintf(_T!(b"Option not implemented: srchlist\n\0".as_ptr() as *const CHAR));
                    }
                    else if _strnicmp(args_vector[1 as usize] as *const CHAR, b"root=\0".as_ptr() as *const CHAR, strlen(b"root=\0".as_ptr() as *const CHAR)) == 0
                    {
                        _tprintf(_T!(b"Option not implemented: root\n\0".as_ptr() as *const CHAR));
                    }
                    else if _strnicmp(args_vector[1 as usize] as *const CHAR, b"type=\0".as_ptr() as *const CHAR, strlen(b"type=\0".as_ptr() as *const CHAR)) == 0
                    {
                        if strncmp(TypeA.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypeA.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeA.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeAAAA.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypeAAAA.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeAAAA.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeBoth.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypeBoth.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeBoth.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeAny.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypeAny.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeAny.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeCNAME.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypeCNAME.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeCNAME.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeMX.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypeMX.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeMX.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeNS.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypeNS.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeNS.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypePTR.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypePTR.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypePTR.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeSOA.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypeSOA.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeSOA.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeSRV.as_ptr() as *const CHAR, args_vector[1 as usize].add(6 as usize) as *const CHAR, strlen(TypeSRV.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeSRV.as_ptr() as PCHAR;
                        }
                        else
                        {
                            _tprintf(_T!(b"unknown query type: %s\0".as_ptr() as *const CHAR), args_vector[1 as usize].add(6 as usize));
                        }
                    }
                    else if _strnicmp(args_vector[1 as usize] as *const CHAR, b"querytype=\0".as_ptr() as *const CHAR, strlen(b"querytype=\0".as_ptr() as *const CHAR)) == 0
                    {
                        if strncmp(TypeA.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypeA.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeA.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeAAAA.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypeAAAA.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeAAAA.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeBoth.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypeBoth.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeBoth.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeAny.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypeAny.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeAny.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeCNAME.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypeCNAME.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeCNAME.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeMX.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypeMX.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeMX.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeNS.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypeNS.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeNS.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypePTR.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypePTR.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypePTR.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeSOA.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypeSOA.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeSOA.as_ptr() as PCHAR;
                        }
                        else if strncmp(TypeSRV.as_ptr() as *const CHAR, args_vector[1 as usize].add(11 as usize) as *const CHAR, strlen(TypeSRV.as_ptr() as *const CHAR)) == 0
                        {
                            State.type_ = TypeSRV.as_ptr() as PCHAR;
                        }
                        else
                        {
                            _tprintf(_T!(b"unknown query type: %s\0".as_ptr() as *const CHAR), args_vector[1 as usize].add(11 as usize));
                        }
                    }
                    else if _strnicmp(args_vector[1 as usize] as *const CHAR, b"class=\0".as_ptr() as *const CHAR, strlen(b"class=\0".as_ptr() as *const CHAR)) == 0
                    {
                        if strncmp(ClassIN.as_ptr() as *const CHAR, args_vector[1 as usize].add(7 as usize) as *const CHAR, strlen(ClassIN.as_ptr() as *const CHAR)) == 0
                        {
                            State.Class = ClassIN.as_ptr() as PCHAR;
                        }
                        else if strncmp(ClassAny.as_ptr() as *const CHAR, args_vector[1 as usize].add(7 as usize) as *const CHAR, strlen(ClassAny.as_ptr() as *const CHAR)) == 0
                        {
                            State.Class = ClassAny.as_ptr() as PCHAR;
                        }
                        else
                        {
                            _tprintf(_T!(b"unknown query class: %s\0".as_ptr() as *const CHAR), args_vector[1 as usize].add(7 as usize));
                        }
                    }
                    else if _strnicmp(args_vector[1 as usize] as *const CHAR, b"ixfrver=\0".as_ptr() as *const CHAR, strlen(b"ixfrver=\0".as_ptr() as *const CHAR)) == 0
                    {
                        _tprintf(_T!(b"Option not implemented: ixfrver\n\0".as_ptr() as *const CHAR));
                    }
                    else
                    {
                        _tprintf(_T!(b"*** Invalid option: %s.\n\0".as_ptr() as *const CHAR), args_vector[1 as usize]);
                    }
                }
            }
            else if _stricmp(args_vector[0 as usize] as *const CHAR, b"server\0".as_ptr() as *const CHAR) == 0
            {
                _tprintf(_T!(b"Command not implemented: server\n\0".as_ptr() as *const CHAR));
            }
            else if _stricmp(args_vector[0 as usize] as *const CHAR, b"lserver\0".as_ptr() as *const CHAR) == 0
            {
                _tprintf(_T!(b"Command not implemented: lserver\n\0".as_ptr() as *const CHAR));
            }
            else if _stricmp(args_vector[0 as usize] as *const CHAR, b"finger\0".as_ptr() as *const CHAR) == 0
            {
                _tprintf(_T!(b"Command not implemented: finger\n\0".as_ptr() as *const CHAR));
            }
            else if _stricmp(args_vector[0 as usize] as *const CHAR, b"root\0".as_ptr() as *const CHAR) == 0
            {
                _tprintf(_T!(b"Command not implemented: root\n\0".as_ptr() as *const CHAR));
            }
            else if _stricmp(args_vector[0 as usize] as *const CHAR, b"ls\0".as_ptr() as *const CHAR) == 0
            {
                _tprintf(_T!(b"Command not implemented: ls\n\0".as_ptr() as *const CHAR));
            }
            else if _stricmp(args_vector[0 as usize] as *const CHAR, b"view\0".as_ptr() as *const CHAR) == 0
            {
                _tprintf(_T!(b"Command not implemented: view\n\0".as_ptr() as *const CHAR));
            }
            else
            {
                if dwArgCount == 1 as DWORD
                {
                    PerformLookup(args_vector[0 as usize]);
                }
                else if dwArgCount == 2 as DWORD
                {
                    let mut BackupServerAddress: [CHAR; 16] = [0; 16];
                    let mut BackupServer: [CHAR; 256] = [0; 256];

                    CopyMemory!(core::ptr::addr_of_mut!(BackupServerAddress) as *mut CHAR, core::ptr::addr_of_mut!(State.DefaultServerAddress) as *const CHAR, core::mem::size_of::<[CHAR; 16]>());
                    CopyMemory!(core::ptr::addr_of_mut!(BackupServer) as *mut CHAR, core::ptr::addr_of_mut!(State.DefaultServer) as *const CHAR, core::mem::size_of::<[CHAR; 256]>());

                    if IsValidIP(args_vector[1 as usize]) != 0
                    {
                        strncpy((core::ptr::addr_of_mut!(State.DefaultServerAddress) as *mut CHAR).cast(), (args_vector[1 as usize] as *const CHAR).cast(), min!(16 as usize, strlen(args_vector[1 as usize] as *const CHAR)));
                        State.DefaultServerAddress[min!(16 as usize, strlen(args_vector[1 as usize] as *const CHAR))] = '\0' as CHAR;

                        PerformInternalLookup(core::ptr::addr_of_mut!(State.DefaultServerAddress) as PCHAR,
                                              core::ptr::addr_of_mut!(State.DefaultServer) as PCHAR);
                    }
                    else
                    {
                        strncpy((core::ptr::addr_of_mut!(State.DefaultServer) as *mut CHAR).cast(), (args_vector[1 as usize] as *const CHAR).cast(), min!(255 as usize, strlen(args_vector[1 as usize] as *const CHAR)));
                        State.DefaultServer[min!(255 as usize, strlen(args_vector[1 as usize] as *const CHAR))] = '\0' as CHAR;

                        PerformInternalLookup(core::ptr::addr_of_mut!(State.DefaultServer) as PCHAR,
                                              core::ptr::addr_of_mut!(State.DefaultServerAddress) as PCHAR);
                    }

                    PerformLookup(args_vector[0 as usize]);

                    CopyMemory!(core::ptr::addr_of_mut!(State.DefaultServerAddress) as *mut CHAR, core::ptr::addr_of_mut!(BackupServerAddress) as *const CHAR, core::mem::size_of::<[CHAR; 16]>());
                    CopyMemory!(core::ptr::addr_of_mut!(State.DefaultServer) as *mut CHAR, core::ptr::addr_of_mut!(BackupServer) as *const CHAR, core::mem::size_of::<[CHAR; 256]>());
                }
                else
                {
                    _tprintf(_T!(b"Unrecognized command:\0".as_ptr() as *const CHAR));
                    let mut i: DWORD = 0 as DWORD;
                    while i < dwArgCount
                    {
                        _tprintf(_T!(b" %s\0".as_ptr() as *const CHAR), args_vector[i as usize]);
                        i = i.wrapping_add(1);
                    }
                    _tprintf(_T!(b"\n\0".as_ptr() as *const CHAR));
                }
            }
        }

        if bDone != 0 {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn main(argc: core::ffi::c_int, argv: *mut *mut core::ffi::c_char) -> core::ffi::c_int {
    let mut i: core::ffi::c_int;
    let mut Status: ULONG;
    let mut pNetInfo: PFIXED_INFO = core::ptr::null_mut();
    let mut NetBufLen: ULONG = 0;
    let mut wsaData: WSADATA = core::mem::zeroed();
    let mut ret: core::ffi::c_int;

    ProcessHeap = GetProcessHeap();
    RequestID = 1;

    /* Set up the initial state. */
    State.debug = (FALSE as i32);
    State.defname = (TRUE as i32);
    State.search = (TRUE as i32);
    State.recurse = (TRUE as i32);
    State.d2 = (FALSE as i32);
    State.vc = (FALSE as i32);
    State.ignoretc = (FALSE as i32);
    State.port = 53;
    State.type_ = TypeBoth.as_ptr() as PCHAR;
    State.Class = ClassIN.as_ptr() as PCHAR;
    State.timeout = 2;
    State.retry = 1;
    State.MSxfr = (TRUE as i32);
    State.ixfrver = 1;

    RtlZeroMemory!(core::ptr::addr_of_mut!(State.root) as *mut core::ffi::c_void, 256);
    RtlZeroMemory!(core::ptr::addr_of_mut!(State.domain) as *mut core::ffi::c_void, 256);
    i = 0;
    while i < 6 {
        RtlZeroMemory!(core::ptr::addr_of_mut!(State.srchlist[i as usize]) as *mut core::ffi::c_void, 256);
        i += 1;
    }
    RtlZeroMemory!(core::ptr::addr_of_mut!(State.DefaultServer) as *mut core::ffi::c_void, 256);
    RtlZeroMemory!(core::ptr::addr_of_mut!(State.DefaultServerAddress) as *mut core::ffi::c_void, 16);

    memcpy(
        (core::ptr::addr_of_mut!(State.root) as *mut core::ffi::c_char).cast(),
        DEFAULT_ROOT.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of_val(&DEFAULT_ROOT),
    );

    /* We don't know how long of a buffer it will want to return. So we'll
       pass an empty one now and let it fail only once, instead of guessing. */
    Status = GetNetworkParams(pNetInfo, core::ptr::addr_of_mut!(NetBufLen));

    if Status != ERROR_BUFFER_OVERFLOW {
        let msg1 = [
            'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ' ' as u16,
            'i' as u16, 'n' as u16, ' ' as u16,
            'G' as u16, 'e' as u16, 't' as u16, 'N' as u16, 'e' as u16, 't' as u16,
            'w' as u16, 'o' as u16, 'r' as u16, 'k' as u16, 'P' as u16, 'a' as u16,
            'r' as u16, 'a' as u16, 'm' as u16, 's' as u16, ' ' as u16,
            'c' as u16, 'a' as u16, 'l' as u16, 'l' as u16, '\n' as u16, 0,
        ];
        _tprintf((msg1.as_ptr()).cast());

        return -2;
    }

    pNetInfo = HeapAlloc(ProcessHeap, 0, (NetBufLen) as SIZE_T) as PFIXED_INFO;
    if pNetInfo.is_null() {
        let msg2 = [
            'E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'R' as u16, ':' as u16, ' ' as u16,
            'O' as u16, 'u' as u16, 't' as u16, ' ' as u16,
            'o' as u16, 'f' as u16, ' ' as u16,
            'm' as u16, 'e' as u16, 'm' as u16, 'o' as u16, 'r' as u16, 'y' as u16, '\n' as u16, 0,
        ];
        _tprintf((msg2.as_ptr()).cast());

        return -1;
    }

    /* For real this time. */
    Status = GetNetworkParams(pNetInfo, core::ptr::addr_of_mut!(NetBufLen));
    if Status != NO_ERROR {
        let msg3 = [
            'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ' ' as u16,
            'i' as u16, 'n' as u16, ' ' as u16,
            'G' as u16, 'e' as u16, 't' as u16, 'N' as u16, 'e' as u16, 't' as u16,
            'w' as u16, 'o' as u16, 'r' as u16, 'k' as u16, 'P' as u16, 'a' as u16,
            'r' as u16, 'a' as u16, 'm' as u16, 's' as u16, ' ' as u16,
            'c' as u16, 'a' as u16, 'l' as u16, 'l' as u16, '\n' as u16, 0,
        ];
        _tprintf((msg3.as_ptr()).cast());

        HeapFree(ProcessHeap, 0, pNetInfo as *mut core::ffi::c_void);

        return -2;
    }

    strncpy(
        (core::ptr::addr_of_mut!(State.domain) as *mut core::ffi::c_char).cast(),
        (core::ptr::addr_of_mut!((*pNetInfo).DomainName) as *const core::ffi::c_char).cast(),
        255,
    );
    strncpy(
        (core::ptr::addr_of_mut!(State.srchlist[0 as usize]) as *mut core::ffi::c_char).cast(),
        (core::ptr::addr_of_mut!((*pNetInfo).DomainName) as *const core::ffi::c_char).cast(),
        255,
    );
    strncpy(
        (core::ptr::addr_of_mut!(State.DefaultServerAddress) as *mut core::ffi::c_char).cast(),
        (core::ptr::addr_of_mut!((*pNetInfo).DnsServerList.IpAddress.String) as *const core::ffi::c_char).cast(),
        15,
    );

    HeapFree(ProcessHeap, 0, pNetInfo as *mut core::ffi::c_void);

    ret = WSAStartup(
        MAKEWORD!(2, 2),
        core::ptr::addr_of_mut!(wsaData),
    );
    if ret != 0 {
        let msg5 = [
            'W' as u16, 'i' as u16, 'n' as u16, 's' as u16, 'o' as u16, 'c' as u16, 'k' as u16, ' ' as u16,
            'i' as u16, 'n' as u16, 'i' as u16, 't' as u16, 'i' as u16, 'a' as u16, 'l' as u16,
            'i' as u16, 'z' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16,
            'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'd' as u16, ':' as u16, ' ' as u16,
            '%' as u16, 'd' as u16, '\n' as u16, 0,
        ];
        _tprintf((msg5.as_ptr()).cast(), ret);
        return ret;
    }

    match ParseCommandLine(argc, argv) {
        0 => {
            /* This means that it was a /? parameter. */
        }
        _ => {
            /* Anything else means we enter interactive mode. The only exception
               to this is when the host to resolve was provided on the command
               line. */
            InteractiveMode();
        }
    }

    WSACleanup();
    return 0;
}
