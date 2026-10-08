use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
use winsrv_sys::RTL_CONSTANT_STRING;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// C-runtime helpers not in the winsrv-sys NTAPI surface. These are
// truly cdecl C-runtime entry points (msvcrt/ntdll exports) — not
// NTAPI — so `extern "C"` is correct here.
extern "C" {
    pub fn wcslen(s: *const u16) -> usize;
}

/// `MAKELANGID(p, s) = ((((WORD)(s)) << 10) | (WORD)(p))` —
/// winnt.h macro.
#[inline]
fn MAKELANGID(p: u16, s: u16) -> u16 {
    ((s as u16) << 10) | (p as u16)
}

/// `MAKELCID(lgid, srtid) = ((DWORD)(((WORD)(srtid)) << 16) | (WORD)(lgid))`
/// — winnt.h macro.
#[inline]
fn MAKELCID(lgid: u16, srtid: u16) -> u32 {
    ((srtid as u32) << 16) | (lgid as u32)
}

/// `HandleToUlong(h) = (ULONG)(ULONG_PTR)(h)` — basetsd.h macro.
#[inline]
fn HandleToUlong(h: *mut core::ffi::c_void) -> u32 {
    h as usize as u32
}

extern "system" {
    pub fn RtlUnicodeStringToAnsiSize(SourceString: *const _UNICODE_STRING) -> u32;
    pub fn RtlStringCbCopyNW( pszDest: *mut u16, cbDest: usize, pszSrc: *const u16, cbSrc: usize, ) -> i32;
}

extern "C" {
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32;
}

// `RtlStringCbPrintfW` and `RtlStringCbPrintfExW` are NTAPI-cdecl
// variadic (C va_list). Rust stable disallows user-defined variadic
// `extern "C" fn` definitions, so we declare them WITH the `...`
// inside `extern "C" {}` blocks (which IS stable for IMPORTS, not
// definitions). The call sites pack up to 5 ULONG_PTR-sized args.
extern "C" {
    pub fn RtlStringCbPrintfW(
        pszDest: PWSTR,
        cbDest: usize,
        pszFormat: PCWSTR,
        ...
    ) -> i32;
    pub fn RtlStringCbPrintfExW(
        pszDest: PWSTR,
        cbDest: usize,
        ppszDestEnd: *mut PWSTR,
        pcbRemaining: *mut usize,
        dwFlags: u32,
        pszFormat: PCWSTR,
        ...
    ) -> i32;
}

// `_scwprintf(fmt, ...)` returns the wide-char count that would be
// written for the formatted string. Rust stable can't host variadic
// `extern "C" fn` definitions, so we deliberately stub it via a
// 4- and 5-positional-arg shim that conservatively over-estimates
// (wcslen(fmt) + 256). Call sites also `* sizeof(u16)` the result
// to size a buffer and pass the same args to `RtlStringCbPrintfW`
// which truncates to fit — a short buffer is a write-truncation,
// not UB. Memory: feedback_degraded_variadic_stubs.
#[inline]
unsafe fn _scwprintf_5(
    format: *const u16,
    _a: ULONG_PTR, _b: ULONG_PTR, _c: ULONG_PTR, _d: ULONG_PTR,
) -> i32 {
    (wcslen(format) as i32).saturating_add(256)
}

#[inline]
unsafe fn _scwprintf(
    format: *const u16,
    _e: i32,
    _a: ULONG_PTR, _b: ULONG_PTR, _c: ULONG_PTR, _d: ULONG_PTR,
) -> i32 {
    (wcslen(format) as i32).saturating_add(256)
}

// ---- file-scope preamble (translated) ----
pub static mut g_CurrentUserLangId: LANGID = 0;
pub static mut g_SuccessU: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
pub static mut g_InformationU: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
pub static mut g_WarningU: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
pub static mut g_ErrorU: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
pub static mut g_SystemProcessU: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
pub static mut g_OKTerminateU: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
pub static mut g_CancelDebugU: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };

unsafe fn RtlLoadUnicodeString(
    hInstance: HINSTANCE,
    uID: u32,
    pUnicodeString: PUNICODE_STRING,
    pDefaultString: PCWSTR,
) {
    let Length: core::ffi::c_int = LoadStringW(
        hInstance,
        uID,
        core::ptr::addr_of_mut!((*pUnicodeString).Buffer) as LPWSTR,
        0,
    );
    if Length == 0 {
        RtlInitUnicodeString(pUnicodeString, pDefaultString);
    } else {
        (*pUnicodeString).MaximumLength = ((Length as u32) * core::mem::size_of::<u16>() as u32) as u16;
        (*pUnicodeString).Length = (*pUnicodeString).MaximumLength;
    }
}

// _scwprintf is defined as an inline unsafe fn at the top of the
// file (Rust-stable doesn't allow user-defined `extern "C"` variadic
// definitions; the inline shim returns a conservative upper bound).

unsafe fn UserpCaptureStringParameters(
    Parameters: PULONG_PTR,
    SizeOfAllUnicodeStrings: *mut u32,
    Message: PHARDERROR_MSG,
    hProcess: *mut core::ffi::c_void,
) {

    let mut Status: i32;
    let mut nParam: u32;
    let mut Size: u32 = 0;
    let mut TempStringU: _UNICODE_STRING = core::mem::zeroed();
    let mut ParamStringU: _UNICODE_STRING = core::mem::zeroed();
    let mut TempStringA: ANSI_STRING = core::mem::zeroed();

    if !SizeOfAllUnicodeStrings.is_null() {
        *SizeOfAllUnicodeStrings = 0;
    }

    nParam = 0;
    while nParam < (*Message).NumberOfParameters {
        *Parameters.offset(nParam as isize) = 0;

        if (*Message).UnicodeStringParameterMask & (1 << nParam) != 0 {
            if hProcess.is_null() {
                nParam += 1;
                continue;
            }

            Status = NtReadVirtualMemory(
                hProcess,
                (*Message).Parameters[nParam as usize] as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(ParamStringU) as *mut core::ffi::c_void,
                core::mem::size_of::<_UNICODE_STRING>() as SIZE_T,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                nParam += 1;
                continue;
            }

            TempStringU.MaximumLength = ParamStringU.Length + core::mem::size_of::<u16>() as u16;
            TempStringU.Length = ParamStringU.Length;
            TempStringU.Buffer = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                TempStringU.MaximumLength as SIZE_T,
            ) as *mut u16;
            if TempStringU.Buffer.is_null() {
                nParam += 1;
                continue;
            }

            Status = NtReadVirtualMemory(
                hProcess,
                ParamStringU.Buffer as *mut core::ffi::c_void,
                TempStringU.Buffer as *mut core::ffi::c_void,
                ParamStringU.Length as SIZE_T,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                RtlFreeHeap(RtlGetProcessHeap(), 0, TempStringU.Buffer as *mut core::ffi::c_void);
                nParam += 1;
                continue;
            }

            *TempStringU.Buffer.offset((TempStringU.Length / core::mem::size_of::<u16>() as u16) as isize) = 0;

            if (*Message).Status == 0x40000018u32 as i32 {
                *Parameters.offset(nParam as isize) = TempStringU.Buffer as ULONG_PTR;
                Size += TempStringU.Length as u32;
            } else {
                TempStringA.MaximumLength = RtlUnicodeStringToAnsiSize(core::ptr::addr_of!(TempStringU)) as u16;
                TempStringA.Buffer = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    HEAP_ZERO_MEMORY,
                    TempStringA.MaximumLength as SIZE_T,
                ) as *mut core::ffi::c_char;
                if TempStringA.Buffer.is_null() {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, TempStringU.Buffer as *mut core::ffi::c_void);
                    nParam += 1;
                    continue;
                }

                Status = RtlUnicodeStringToAnsiString(
                    core::ptr::addr_of_mut!(TempStringA),
                    core::ptr::addr_of!(TempStringU),
                    0u8,
                );
                RtlFreeHeap(RtlGetProcessHeap(), 0, TempStringU.Buffer as *mut core::ffi::c_void);
                if !NT_SUCCESS(Status) {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, TempStringA.Buffer as *mut core::ffi::c_void);
                    nParam += 1;
                    continue;
                }

                *Parameters.offset(nParam as isize) = TempStringA.Buffer as ULONG_PTR;
                Size += TempStringU.Length as u32;
            }
        } else {
            *Parameters.offset(nParam as isize) = (*Message).Parameters[nParam as usize];
        }

        nParam += 1;
    }

    if !SizeOfAllUnicodeStrings.is_null() {
        *SizeOfAllUnicodeStrings = Size;
    }
}

unsafe fn UserpFreeStringParameters(
    Parameters: PULONG_PTR,
    Message: PHARDERROR_MSG,
) {
    let _nParam: u32;

    for nParam in 0..(*Message).NumberOfParameters {
        if (((*Message).UnicodeStringParameterMask & (1 << nParam)) != 0) && (*Parameters.add(nParam as usize) != 0) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, *Parameters.add(nParam as usize) as *mut core::ffi::c_void);
        }
    }
}

unsafe fn UserpGetClientFileName(
    ClientFileNameU: PUNICODE_STRING,
    hProcess: HANDLE,
) -> NTSTATUS {
    let ModuleListHead: PLIST_ENTRY;
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let Module: PLDR_DATA_TABLE_ENTRY;
    let mut Ldr: PPEB_LDR_DATA = core::ptr::null_mut();
    let mut ClientBasicInfo: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let mut ModuleData: LDR_DATA_TABLE_ENTRY = core::mem::zeroed();
    let mut ClientDllBase: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let Peb: PPEB;

    RtlInitEmptyUnicodeString(ClientFileNameU, core::ptr::null_mut(), 0);

    Status = NtQueryInformationProcess(
        hProcess,
        ProcessBasicInformation,
        core::ptr::addr_of_mut!(ClientBasicInfo) as *mut _,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Peb = ClientBasicInfo.PebBaseAddress;
    if Peb.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    Status = NtReadVirtualMemory(
        hProcess,
        core::ptr::addr_of_mut!((*Peb).Ldr) as *mut _,
        core::ptr::addr_of_mut!(Ldr) as *mut _,
        core::mem::size_of::<PPEB_LDR_DATA>() as SIZE_T,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ModuleListHead = core::ptr::addr_of_mut!((*Ldr).InLoadOrderModuleList);
    Status = NtReadVirtualMemory(
        hProcess,
        core::ptr::addr_of_mut!((*ModuleListHead).Flink) as *mut _,
        core::ptr::addr_of_mut!(Entry) as *mut _,
        core::mem::size_of::<PLIST_ENTRY>() as SIZE_T,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if Entry == ModuleListHead {
        return STATUS_UNSUCCESSFUL;
    }

    Module = (Entry as usize - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as PLDR_DATA_TABLE_ENTRY;

    Status = NtReadVirtualMemory(
        hProcess,
        Module as *mut _,
        core::ptr::addr_of_mut!(ModuleData) as *mut _,
        core::mem::size_of::<LDR_DATA_TABLE_ENTRY>() as SIZE_T,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = NtReadVirtualMemory(
        hProcess,
        core::ptr::addr_of_mut!((*Peb).ImageBaseAddress) as *mut _,
        core::ptr::addr_of_mut!(ClientDllBase) as *mut _,
        core::mem::size_of::<PVOID>() as SIZE_T,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if ClientDllBase != ModuleData.DllBase {
        return STATUS_UNSUCCESSFUL;
    }

    (*ClientFileNameU).MaximumLength = ModuleData.BaseDllName.MaximumLength;
    (*ClientFileNameU).Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (*ClientFileNameU).MaximumLength as SIZE_T,
    ) as *mut u16;
    if (*ClientFileNameU).Buffer.is_null() {
        RtlInitEmptyUnicodeString(ClientFileNameU, core::ptr::null_mut(), 0);
        return STATUS_NO_MEMORY;
    }

    Status = NtReadVirtualMemory(
        hProcess,
        ModuleData.BaseDllName.Buffer as *mut _,
        (*ClientFileNameU).Buffer as *mut _,
        (*ClientFileNameU).MaximumLength as SIZE_T,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*ClientFileNameU).Buffer as *mut _);
        RtlInitEmptyUnicodeString(ClientFileNameU, core::ptr::null_mut(), 0);
        return Status;
    }

    (*ClientFileNameU).Length = (wcslen((*ClientFileNameU).Buffer) * core::mem::size_of::<u16>()) as u16;

    STATUS_SUCCESS
}

unsafe fn UserpDuplicateParamStringToUnicodeString(
    UnicodeString: PUNICODE_STRING,
    ParamString: PCWSTR)
{
    let mut FormatU: _UNICODE_STRING = core::mem::zeroed();
    let mut TempStringU: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut FormatU, ParamString as PWSTR);
    if (*UnicodeString).MaximumLength < FormatU.MaximumLength
    {
        if NT_SUCCESS(RtlDuplicateUnicodeString(RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
                                                 &FormatU, &mut TempStringU))
        {
            *UnicodeString = TempStringU;
        }
        else
        {
            FormatU.MaximumLength = (*UnicodeString).MaximumLength;
            FormatU.Length = FormatU.MaximumLength - core::mem::size_of::<u16>() as u16;
            RtlCopyUnicodeString(UnicodeString, &FormatU);
        }
    }
    else
    {
        RtlCopyUnicodeString(UnicodeString, &FormatU);
    }
}

unsafe fn UserpFormatMessages(
    TextStringU: PUNICODE_STRING,
    CaptionStringU: PUNICODE_STRING,
    pdwType: PUINT,
    pdwTimeout: *mut u32,
    Message: PHARDERROR_MSG,
) {

    const RT_MESSAGETABLE: u32 = 11;

    static pszUnknownHardError: &[u16] = &[
        'U' as u16, 'n' as u16, 'k' as u16, 'n' as u16, 'o' as u16, 'w' as u16, 'n' as u16, ' ' as u16,
        'H' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16, 'E' as u16, 'r' as u16, 'r' as u16,
        'o' as u16, 'r' as u16, ' ' as u16, '0' as u16, 'x' as u16, '%' as u16, '0' as u16, '8' as u16,
        'l' as u16, 'x' as u16, '\n' as u16, 'P' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 'm' as u16,
        'e' as u16, 't' as u16, 'e' as u16, 'r' as u16, 's' as u16, ':' as u16, ' ' as u16, '0' as u16,
        'x' as u16, '%' as u16, 'p' as u16, ' ' as u16, '0' as u16, 'x' as u16, '%' as u16, 'p' as u16,
        ' ' as u16, '0' as u16, 'x' as u16, '%' as u16, 'p' as u16, ' ' as u16, '0' as u16, 'x' as u16,
        '%' as u16, 'p' as u16, 0,
    ];

    static pszExceptionHardError: &[u16] = &[
        'E' as u16, 'x' as u16, 'c' as u16, 'e' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16,
        'n' as u16, ' ' as u16, 'p' as u16, 'r' as u16, 'o' as u16, 'c' as u16, 'e' as u16, 's' as u16,
        's' as u16, 'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16, 'm' as u16, 'e' as u16, 's' as u16,
        's' as u16, 'a' as u16, 'g' as u16, 'e' as u16, ' ' as u16, '0' as u16, 'x' as u16, '%' as u16,
        '0' as u16, '8' as u16, 'l' as u16, 'x' as u16, '\n' as u16, 'P' as u16, 'a' as u16, 'r' as u16,
        'a' as u16, 'm' as u16, 'e' as u16, 't' as u16, 'e' as u16, 'r' as u16, 's' as u16, ':' as u16,
        ' ' as u16, '0' as u16, 'x' as u16, '%' as u16, 'p' as u16, ' ' as u16, '0' as u16, 'x' as u16,
        '%' as u16, 'p' as u16, ' ' as u16, '0' as u16, 'x' as u16, '%' as u16, 'p' as u16, ' ' as u16,
        '0' as u16, 'x' as u16, '%' as u16, 'p' as u16, 0,
    ];

    let mut Status: i32;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hProcess: *mut core::ffi::c_void = core::ptr::null_mut();
    let Severity: u32 = ((*Message).Status as u32) >> 30;
    let mut SizeOfStrings: u32 = 0;
    let mut Parameters: [ULONG_PTR; MAXIMUM_HARDERROR_PARAMETERS as usize] = [0; MAXIMUM_HARDERROR_PARAMETERS as usize];
    let mut CopyParameters: [ULONG_PTR; MAXIMUM_HARDERROR_PARAMETERS as usize] = [0; MAXIMUM_HARDERROR_PARAMETERS as usize];
    let mut WindowTitleU: _UNICODE_STRING = core::mem::zeroed();
    let mut FileNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut TempStringU: _UNICODE_STRING = core::mem::zeroed();
    let mut FormatU: _UNICODE_STRING = core::mem::zeroed();
    let mut Format2U: _UNICODE_STRING = core::mem::zeroed();
    let mut FormatA: ANSI_STRING = core::mem::zeroed();
    let mut Format2A: ANSI_STRING = core::mem::zeroed();
    let mut hwndOwner: HWND;
    let mut MessageResource: PMESSAGE_RESOURCE_ENTRY = core::ptr::null_mut();
    let mut FormatString: PWSTR;
    let mut pszBuffer: PWSTR;
    let mut cszBuffer: usize;

    core::ptr::write_bytes(core::ptr::addr_of_mut!(ObjectAttributes), 0, 1);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = core::ptr::null_mut();
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenProcess(
        &mut hProcess,
        PROCESS_VM_READ | PROCESS_QUERY_INFORMATION,
        &mut ObjectAttributes,
        core::ptr::addr_of_mut!((*Message).h.__bindgen_anon_1.ClientId),
    );
    if !NT_SUCCESS(Status) {
        hProcess = core::ptr::null_mut();
    }

    UserpCaptureStringParameters(Parameters.as_mut_ptr(), &mut SizeOfStrings, Message, hProcess);

    (*TextStringU).Length = 0;
    (*TextStringU).Buffer.write(0);

    (*CaptionStringU).Length = 0;
    (*CaptionStringU).Buffer.write(0);

    if (*Message).Status == 0x40000018 {
        if !hProcess.is_null() {
            NtClose(hProcess);
        }

        *pdwType = (Parameters[2] as u32) & !MB_SERVICE_NOTIFICATION;

        if (*Message).UnicodeStringParameterMask & 0x1 != 0 {
            UserpDuplicateParamStringToUnicodeString(TextStringU, Parameters[0] as PCWSTR);
        }
        if (*Message).UnicodeStringParameterMask & 0x2 != 0 {
            UserpDuplicateParamStringToUnicodeString(CaptionStringU, Parameters[1] as PCWSTR);
        }

        if (*Message).NumberOfParameters >= 4 {
            *pdwTimeout = Parameters[3] as u32;
        } else {
            *pdwTimeout = INFINITE;
        }

        UserpFreeStringParameters(Parameters.as_mut_ptr(), Message);
        return;
    }

    *pdwType = 0;
    match (*Message).ValidResponseOptions {
        OptionAbortRetryIgnore => {
            *pdwType = MB_ABORTRETRYIGNORE;
        }
        OptionOk => {
            *pdwType = MB_OK;
        }
        OptionOkCancel => {
            *pdwType = MB_OKCANCEL;
        }
        OptionRetryCancel => {
            *pdwType = MB_RETRYCANCEL;
        }
        OptionYesNo => {
            *pdwType = MB_YESNO;
        }
        OptionYesNoCancel => {
            *pdwType = MB_YESNOCANCEL;
        }
        OptionShutdownSystem => {
            *pdwType = MB_OK;
        }
        OptionOkNoWait => {
            *pdwType = MB_OK;
        }
        OptionCancelTryContinue => {
            *pdwType = MB_CANCELTRYCONTINUE;
        }
        _ => {}
    }

    if Severity == STATUS_SEVERITY_INFORMATIONAL {
        *pdwType |= MB_ICONINFORMATION;
    } else if Severity == STATUS_SEVERITY_WARNING {
        *pdwType |= MB_ICONWARNING;
    } else if Severity == STATUS_SEVERITY_ERROR {
        *pdwType |= MB_ICONERROR;
    }

    *pdwType |= MB_SYSTEMMODAL | MB_SETFOREGROUND;

    *pdwTimeout = INFINITE;

    CopyParameters = Parameters;

    Status = STATUS_SUCCESS;
    if !hProcess.is_null() {
        Status = UserpGetClientFileName(&mut FileNameU, hProcess);
    }

    if !hProcess.is_null() {
        NtClose(hProcess);
    }

    if hProcess.is_null() || !NT_SUCCESS(Status) || FileNameU.Buffer.is_null() {
        hProcess = core::ptr::null_mut();
        FileNameU = g_SystemProcessU;
    }

    FormatA.Buffer = core::ptr::null_mut();
    Status = RtlFindMessage(
        GetModuleHandleW(w!("ntdll").as_ptr() as PCWSTR) as PVOID,
        RT_MESSAGETABLE,
        MAKELANGID(LANG_NEUTRAL as u16, SUBLANG_NEUTRAL as u16) as u32,
        (*Message).Status as u32,
        &mut MessageResource,
    );
    if NT_SUCCESS(Status) {
        if (*MessageResource).Flags != 0 {
            RtlInitUnicodeString(&mut FormatU, (*MessageResource).Text.as_ptr() as PWSTR);
            FormatA.Buffer = core::ptr::null_mut();
        } else {
            RtlInitAnsiString(&mut FormatA, (*MessageResource).Text.as_ptr() as PSTR);
            RtlAnsiStringToUnicodeString(&mut FormatU, &mut FormatA, 1);
        }
    } else {
        RtlInitEmptyUnicodeString(&mut FormatU, core::ptr::null_mut(), 0);
        FormatA.Buffer = core::ptr::null_mut();
    }

    FormatString = FormatU.Buffer;

    if !FormatString.is_null() && FormatString.read() == '{' as u16 {
        TempStringU.Buffer = FormatString.offset(1);
        FormatString = FormatString.offset(1);

        TempStringU.Length = 0;
        while FormatString.read() != 0 && FormatString.read() != '}' as u16 {
            TempStringU.Length += 1;
            FormatString = FormatString.offset(1);
        }

        FormatString = FormatString.offset(3);

        TempStringU.Length *= core::mem::size_of::<u16>() as u16;
        TempStringU.MaximumLength = TempStringU.Length;
    } else {
        if Severity == STATUS_SEVERITY_SUCCESS {
            TempStringU = g_SuccessU;
        } else if Severity == STATUS_SEVERITY_INFORMATIONAL {
            TempStringU = g_InformationU;
        } else if Severity == STATUS_SEVERITY_WARNING {
            TempStringU = g_WarningU;
        } else if Severity == STATUS_SEVERITY_ERROR {
            TempStringU = g_ErrorU;
        } else {
            TempStringU = g_ErrorU;
        }
    }

    RtlInitEmptyUnicodeString(&mut WindowTitleU, w!("").as_ptr() as PWSTR, 0);
    hwndOwner = core::ptr::null_mut();
    // FindTopLevelWnd: bindgen-declared (extern "stdcall") + Rust-side
    // definition in init.rs (extern "system"). i386 codegen identical;
    // we go through the bindgen FnDef to satisfy the Option<stdcall>
    // type of EnumThreadWindows. Linker resolves to the Rust impl.
    EnumThreadWindows(
        HandleToUlong((*Message).h.__bindgen_anon_1.ClientId.UniqueThread),
        Some(FindTopLevelWnd),
        core::ptr::addr_of_mut!(hwndOwner) as LPARAM,
    );
    if !hwndOwner.is_null() {
        cszBuffer = GetWindowTextLengthW(hwndOwner) as usize;
        if cszBuffer != 0 {
            cszBuffer += 3;
            cszBuffer *= core::mem::size_of::<u16>();
            pszBuffer = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, cszBuffer as SIZE_T) as PWSTR;
            if !pszBuffer.is_null() {
                RtlInitEmptyUnicodeString(&mut WindowTitleU, pszBuffer, cszBuffer as u16);
                cszBuffer = GetWindowTextW(
                    hwndOwner,
                    WindowTitleU.Buffer,
                    WindowTitleU.MaximumLength as i32 / core::mem::size_of::<u16>() as i32,
                ) as usize;
                WindowTitleU.Length = (cszBuffer * core::mem::size_of::<u16>()) as u16;
                RtlAppendUnicodeToString(&mut WindowTitleU, w!(": ").as_ptr() as PCWSTR);
            }
        }
    }

    cszBuffer = WindowTitleU.Length as usize + FileNameU.Length as usize + TempStringU.Length as usize +
                3 * core::mem::size_of::<u16>() + core::mem::size_of::<u16>();
    if (*CaptionStringU).MaximumLength < cszBuffer as u16 {
        pszBuffer = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, cszBuffer as SIZE_T) as PWSTR;
        if pszBuffer.is_null() {
        } else {
            RtlInitEmptyUnicodeString(CaptionStringU, pszBuffer, cszBuffer as u16);
        }
    }
    (*CaptionStringU).Length = 0;
    (*CaptionStringU).Buffer.write(0);

    RtlStringCbPrintfW(
        (*CaptionStringU).Buffer,
        (*CaptionStringU).MaximumLength as usize,
        w!("%wZ%wZ - %wZ").as_ptr() as PCWSTR,
        &WindowTitleU,
        &FileNameU,
        &TempStringU,
    );
    (*CaptionStringU).Length = (wcslen((*CaptionStringU).Buffer) * core::mem::size_of::<u16>()) as u16;

    if !WindowTitleU.Buffer.is_null() && WindowTitleU.MaximumLength != 0 {
        RtlFreeUnicodeString(&mut WindowTitleU);
    }
    if !hProcess.is_null() {
        RtlFreeUnicodeString(&mut FileNameU);
    }

    Format2A.Buffer = core::ptr::null_mut();

    if FormatString.is_null() {
        'BuildMessage: loop {
            cszBuffer = 0;
            cszBuffer = _scwprintf(
                pszUnknownHardError.as_ptr(),
                (*Message).Status,
                Parameters[0],
                Parameters[1],
                Parameters[2],
                Parameters[3],
            ) as usize;
            cszBuffer *= core::mem::size_of::<u16>();

            cszBuffer += SizeOfStrings as usize + core::mem::size_of::<u16>();

            if (*TextStringU).MaximumLength < cszBuffer as u16 {
                pszBuffer = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, cszBuffer as SIZE_T) as PWSTR;
                if pszBuffer.is_null() {
                } else {
                    RtlInitEmptyUnicodeString(TextStringU, pszBuffer, cszBuffer as u16);
                }
            }
            (*TextStringU).Length = 0;
            (*TextStringU).Buffer.write(0);

            pszBuffer = (*TextStringU).Buffer;
            cszBuffer = (*TextStringU).MaximumLength as usize;

            RtlStringCbPrintfW(
                pszBuffer,
                cszBuffer,
                pszUnknownHardError.as_ptr(),
                (*Message).Status,
                Parameters[0],
                Parameters[1],
                Parameters[2],
                Parameters[3],
            );

            (*TextStringU).Length = (wcslen((*TextStringU).Buffer) * core::mem::size_of::<u16>()) as u16;

            if !Format2A.Buffer.is_null() {
                RtlFreeUnicodeString(&mut Format2U);
            }
            if !FormatA.Buffer.is_null() {
                RtlFreeUnicodeString(&mut FormatU);
            }

            UserpFreeStringParameters(Parameters.as_mut_ptr(), Message);
            return;
        }
    }

    if (*Message).Status == STATUS_UNHANDLED_EXCEPTION {
        let ExceptionCode: u32 = CopyParameters[0] as u32;

        Status = RtlFindMessage(
            GetModuleHandleW(w!("ntdll").as_ptr() as PCWSTR) as PVOID,
            RT_MESSAGETABLE,
            MAKELANGID(LANG_NEUTRAL as u16, SUBLANG_NEUTRAL as u16) as u32,
            ExceptionCode,
            &mut MessageResource,
        );
        if NT_SUCCESS(Status) {
            if (*MessageResource).Flags != 0 {
                RtlInitUnicodeString(&mut Format2U, (*MessageResource).Text.as_ptr() as PWSTR);
                Format2A.Buffer = core::ptr::null_mut();
            } else {
                RtlInitAnsiString(&mut Format2A, (*MessageResource).Text.as_ptr() as PSTR);
                RtlAnsiStringToUnicodeString(&mut Format2U, &mut Format2A, 1);
            }

            if ExceptionCode == 0xC0000005 {
                FormatString = Format2U.Buffer;
                CopyParameters[0] = CopyParameters[1];
                CopyParameters[1] = CopyParameters[3];
                if CopyParameters[2] != 0 {
                    CopyParameters[2] = b"w\0r\0i\0t\0t\0e\0n\0\0\0".as_ptr() as ULONG_PTR;
                } else {
                    CopyParameters[2] = b"r\0e\0a\0d\0\0\0".as_ptr() as ULONG_PTR;
                }
            } else if ExceptionCode == 0xC0000006 {
                FormatString = Format2U.Buffer;
                CopyParameters[0] = CopyParameters[1];
                CopyParameters[1] = CopyParameters[3];
            } else {
                CopyParameters[2] = CopyParameters[1];
                CopyParameters[1] = CopyParameters[0];

                pszBuffer = Format2U.Buffer;
                if _wcsnicmp(pszBuffer, w!("{EXCEPTION}").as_ptr() as *const u16, 11) == 0 {
                    pszBuffer = pszBuffer.offset(11);
                    pszBuffer = pszBuffer.offset(2);
                    CopyParameters[0] = pszBuffer as ULONG_PTR;
                } else {
                    CopyParameters[0] = b"u\0n\0k\0n\0o\0w\0n\0 \0s\0o\0f\0t\0w\0a\0r\0e\0 \0e\0x\0c\0e\0p\0t\0i\0o\0n\0\0\0".as_ptr() as ULONG_PTR;
                }
            }
        } else {
            CopyParameters[2] = CopyParameters[1];
            CopyParameters[1] = CopyParameters[0];
            CopyParameters[0] = b"u\0n\0k\0n\0o\0w\0n\0 \0s\0o\0f\0t\0w\0a\0r\0e\0 \0e\0x\0c\0e\0p\0t\0i\0o\0n\0\0\0".as_ptr() as ULONG_PTR;
        }
    }

    cszBuffer = 0;
    if FormatString.is_null() {
        cszBuffer = _scwprintf(
            pszUnknownHardError.as_ptr(),
            (*Message).Status,
            Parameters[0],
            Parameters[1],
            Parameters[2],
            Parameters[3],
        ) as usize;
        cszBuffer *= core::mem::size_of::<u16>();
    } else {
        cszBuffer = _scwprintf_5(
            FormatString,
            CopyParameters[0],
            CopyParameters[1],
            CopyParameters[2],
            CopyParameters[3],
        ) as usize;
        cszBuffer *= core::mem::size_of::<u16>();

        if (*Message).Status == STATUS_UNHANDLED_EXCEPTION {
            if (*Message).ValidResponseOptions == OptionOk || (*Message).ValidResponseOptions == OptionOkCancel {
                cszBuffer += core::mem::size_of::<u16>() + g_OKTerminateU.Length as usize;
            }
            if (*Message).ValidResponseOptions == OptionOkCancel {
                cszBuffer += core::mem::size_of::<u16>() + g_CancelDebugU.Length as usize;
            }
        }
    }

    cszBuffer += SizeOfStrings as usize + core::mem::size_of::<u16>();

    if (*TextStringU).MaximumLength < cszBuffer as u16 {
        pszBuffer = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, cszBuffer as SIZE_T) as PWSTR;
        if pszBuffer.is_null() {
        } else {
            RtlInitEmptyUnicodeString(TextStringU, pszBuffer, cszBuffer as u16);
        }
    }
    (*TextStringU).Length = 0;
    (*TextStringU).Buffer.write(0);

    pszBuffer = (*TextStringU).Buffer;
    cszBuffer = (*TextStringU).MaximumLength as usize;

    if FormatString.is_null() {
        RtlStringCbPrintfW(
            pszBuffer,
            cszBuffer,
            pszUnknownHardError.as_ptr(),
            (*Message).Status,
            Parameters[0],
            Parameters[1],
            Parameters[2],
            Parameters[3],
        );
    } else {
        RtlStringCbPrintfExW(
            pszBuffer,
            cszBuffer,
            &mut pszBuffer,
            &mut cszBuffer,
            0,
            FormatString,
            CopyParameters[0],
            CopyParameters[1],
            CopyParameters[2],
            CopyParameters[3],
        );

        if (*Message).Status == STATUS_UNHANDLED_EXCEPTION {
            if (*Message).ValidResponseOptions == OptionOk || (*Message).ValidResponseOptions == OptionOkCancel {
                RtlStringCbPrintfExW(
                    pszBuffer,
                    cszBuffer,
                    &mut pszBuffer,
                    &mut cszBuffer,
                    0,
                    w!("\n%wZ").as_ptr() as PCWSTR,
                    &raw const g_OKTerminateU,
                );
            }
            if (*Message).ValidResponseOptions == OptionOkCancel {
                RtlStringCbPrintfExW(
                    pszBuffer,
                    cszBuffer,
                    &mut pszBuffer,
                    &mut cszBuffer,
                    0,
                    w!("\n%wZ").as_ptr() as PCWSTR,
                    &raw const g_CancelDebugU,
                );
            }
        }
    }

    (*TextStringU).Length = (wcslen((*TextStringU).Buffer) * core::mem::size_of::<u16>()) as u16;

    if !Format2A.Buffer.is_null() {
        RtlFreeUnicodeString(&mut Format2U);
    }
    if !FormatA.Buffer.is_null() {
        RtlFreeUnicodeString(&mut FormatU);
    }

    UserpFreeStringParameters(Parameters.as_mut_ptr(), Message);
}

unsafe fn GetRegInt(
    KeyName: PCWSTR,
    ValueName: PCWSTR,
    DefaultValue: u32,
) -> u32 {
    let mut Value = DefaultValue;
    let mut String: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ResultLength: u32 = 0;
    let mut ValueBuffer: [u8; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + core::mem::size_of::<u32>()] = core::mem::zeroed();
    let ValueInfo = ValueBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;

    RtlInitUnicodeString(&mut String, KeyName);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut String;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let Status = NtOpenKey(&mut KeyHandle, KEY_READ, &mut ObjectAttributes);
    if NT_SUCCESS(Status) {
        RtlInitUnicodeString(&mut String, ValueName);
        let Status = NtQueryValueKey(
            KeyHandle,
            &mut String,
            KeyValuePartialInformation,
            ValueInfo as *mut core::ffi::c_void,
            core::mem::size_of_val(&ValueBuffer) as u32,
            &mut ResultLength,
        );

        NtClose(KeyHandle);

        if NT_SUCCESS(Status) && ((*ValueInfo).Type == REG_DWORD) {
            Value = *((*ValueInfo).Data.as_ptr() as *const u32);
        }
    }

    Value
}

unsafe fn UserpShowInformationBalloon(
    TextStringU: PUNICODE_STRING,
    CaptionStringU: PUNICODE_STRING,
    Type: u32,
    Message: PHARDERROR_MSG,
) -> i32 {
    let ShellErrorMode: u32;
    let hWndTaskman: HWND;
    let mut CopyData: COPYDATASTRUCT = core::mem::zeroed();
    let pdata: PBALLOON_HARD_ERROR_DATA;
    let mut dwSize: u32;
    let cbTextLen: u32;
    let cbTitleLen: u32;
    let pText: *mut u16;
    let pCaption: *mut u16;
    let ret: BOOL;
    let mut dwResult: DWORD_PTR;

    ShellErrorMode = GetRegInt(
        w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Windows").as_ptr() as *const u16,
        w!("ShellErrorMode").as_ptr() as *const u16,
        0,
    );

    if ShellErrorMode != 1 {
        return 0;
    }

    hWndTaskman = GetTaskmanWindow();
    if hWndTaskman.is_null() {
        return 0;
    }

    cbTextLen = (*TextStringU).Length as u32 + core::mem::size_of::<u16>() as u32;
    cbTitleLen = (*CaptionStringU).Length as u32 + core::mem::size_of::<u16>() as u32;

    dwSize = core::mem::size_of::<BALLOON_HARD_ERROR_DATA>() as u32;
    dwSize += cbTextLen + cbTitleLen;

    pdata = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, dwSize as SIZE_T) as PBALLOON_HARD_ERROR_DATA;
    if pdata.is_null() {
        return 0;
    }

    (*pdata).cbHeaderSize = core::mem::size_of::<BALLOON_HARD_ERROR_DATA>() as u32;
    (*pdata).Status = (*Message).Status as u32;
    (*pdata).dwType = Type;

    (*pdata).TitleOffset = (*pdata).cbHeaderSize as ULONG_PTR;
    (*pdata).MessageOffset = (*pdata).TitleOffset + cbTitleLen as ULONG_PTR;
    pCaption = (pdata as usize + (*pdata).TitleOffset as usize) as *mut u16;
    pText = (pdata as usize + (*pdata).MessageOffset as usize) as *mut u16;
    RtlStringCbCopyNW(pCaption, cbTitleLen as usize, (*CaptionStringU).Buffer, (*CaptionStringU).Length as usize);
    RtlStringCbCopyNW(pText, cbTextLen as usize, (*TextStringU).Buffer, (*TextStringU).Length as usize);

    CopyData.dwData = RegisterWindowMessageW(w!("HardError").as_ptr() as *const u16) as ULONG_PTR;
    CopyData.cbData = dwSize;
    CopyData.lpData = pdata as *mut core::ffi::c_void;

    dwResult = 0;
    ret = SendMessageTimeoutW(hWndTaskman, WM_COPYDATA, 0, &CopyData as *const _ as LPARAM,
                              SMTO_NORMAL | SMTO_ABORTIFHUNG, 3000, &mut dwResult) as BOOL;

    RtlFreeHeap(RtlGetProcessHeap(), 0, pdata as *mut core::ffi::c_void);

    if ret != 0 && dwResult != 0 { 1 } else { 0 }
}

unsafe fn UserpMessageBox(
    TextStringU: PUNICODE_STRING,
    CaptionStringU: PUNICODE_STRING,
    Type: u32,
    Timeout: u32,
) -> u32 {
    let MessageBoxResponse: i32;

    MessageBoxResponse = MessageBoxTimeoutW(
        core::ptr::null_mut(),
        (*TextStringU).Buffer,
        (*CaptionStringU).Buffer,
        Type,
        0,
        Timeout,
    );

    match MessageBoxResponse as u32 {
        IDOK => return ResponseOk,
        IDCANCEL => return ResponseCancel,
        IDYES => return ResponseYes,
        IDNO => return ResponseNo,
        IDABORT => return ResponseAbort,
        IDIGNORE => return ResponseIgnore,
        IDRETRY => return ResponseRetry,
        IDTRYAGAIN => return ResponseTryAgain,
        IDCONTINUE => return ResponseContinue,
        _ => return ResponseNotHandled,
    }
}

unsafe fn UserpLogHardError(
    TextStringU: PUNICODE_STRING,
    CaptionStringU: PUNICODE_STRING,
) {
    // Event-log RPC reports transport failures by raising an exception.  On
    // arm64 our DUMMY_PSEH build cannot catch that best-effort logging failure,
    // so an unavailable EventLog service would kill the critical csrss process
    // while it is trying to display an unrelated hard error.  Preserve the
    // actual hard-error handling and omit only the optional log copy.
    #[cfg(target_arch = "aarch64")]
    return;

    #[cfg(not(target_arch = "aarch64"))]
    {
    let mut Status: NTSTATUS;
    let mut hEventLog: HANDLE = core::ptr::null_mut();
    let mut UNCServerNameU = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let SourceNameU = RTL_CONSTANT_STRING!(w!("Application Popup").as_ptr());
    let Strings: [PUNICODE_STRING; 2] = [CaptionStringU, TextStringU];

    Status = ElfRegisterEventSourceW(
        &mut UNCServerNameU,
        &SourceNameU as *const _UNICODE_STRING as *mut _UNICODE_STRING,
        &mut hEventLog,
    );
    if !NT_SUCCESS(Status) || hEventLog.is_null() {
        return;
    }

    Status = ElfReportEventW(
        hEventLog,
        EVENTLOG_INFORMATION_TYPE as u16,
        0,
        STATUS_LOG_HARD_ERROR,
        core::ptr::null_mut(),
        2,
        0,
        Strings.as_ptr() as *mut PUNICODE_STRING,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    ElfDeregisterEventSource(hEventLog);
    }
}

// ntstatus.h. The translation wrote `0xC0000374 - 0x40000000` (= 0x80000374, a status nothing
// raises) in all three places the C tests this, so an unhandled exception's message was
// formatted with the raw "The exception %s ..." text and `%s` pointed at the exception
// CODE: any application crash with a hard-error popup took csrss, and so the system, down
// (BUGCHECK 0xF4 on arm64 after lsass crashed, 2026-10-08).
const STATUS_UNHANDLED_EXCEPTION: NTSTATUS = 0xC0000144u32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn UserServerHardError(
    _ThreadData: PCSR_THREAD,
    Message: PHARDERROR_MSG,
) {
    let ErrorMode: u32;
    let mut dwType: u32 = 0;
    let mut Timeout: u32 = INFINITE;
    let mut TextU: _UNICODE_STRING = core::mem::zeroed();
    let mut CaptionU: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalTextBuffer: [u16; 256] = core::mem::zeroed();
    let mut LocalCaptionBuffer: [u16; 256] = core::mem::zeroed();
    let Status: NTSTATUS;

    (*Message).Response = ResponseNotHandled;

    if (*Message).NumberOfParameters > MAXIMUM_HARDERROR_PARAMETERS {
        (*Message).NumberOfParameters = MAXIMUM_HARDERROR_PARAMETERS;
    }
    if (*Message).ValidResponseOptions > OptionCancelTryContinue {
        return;
    }
    if (*Message).Status == STATUS_SERVICE_NOTIFICATION {
        if (*Message).NumberOfParameters < 3 {
            return;
        }
    }

    Status = NtUserSetInformationThread(
        (-2isize) as *mut core::ffi::c_void,
        UserThreadUseActiveDesktop,
        core::ptr::null_mut(),
        0,
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    UserInitHardErrorsCache();

    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(TextU),
        LocalTextBuffer.as_mut_ptr(),
        core::mem::size_of_val(&LocalTextBuffer) as u16,
    );
    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(CaptionU),
        LocalCaptionBuffer.as_mut_ptr(),
        core::mem::size_of_val(&LocalCaptionBuffer) as u16,
    );
    UserpFormatMessages(
        core::ptr::addr_of_mut!(TextU),
        core::ptr::addr_of_mut!(CaptionU),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(Timeout),
        Message,
    );

    UserpLogHardError(
        core::ptr::addr_of_mut!(TextU),
        core::ptr::addr_of_mut!(CaptionU),
    );

    ErrorMode = GetRegInt(
        w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Windows").as_ptr(),
        w!("ErrorMode").as_ptr(),
        0,
    );

    if (*Message).Status != STATUS_SERVICE_NOTIFICATION && ErrorMode != 0 {
        (*Message).Response = ResponseOk;
    } else if (*Message).ValidResponseOptions == OptionOkNoWait {
        (*Message).Response = ResponseOk;
        if UserpShowInformationBalloon(
            core::ptr::addr_of_mut!(TextU),
            core::ptr::addr_of_mut!(CaptionU),
            dwType,
            Message,
        ) != 0
        {
            (*Message).Response = ResponseOk;
        } else {
            (*Message).Response = UserpMessageBox(
                core::ptr::addr_of_mut!(TextU),
                core::ptr::addr_of_mut!(CaptionU),
                dwType,
                Timeout,
            );
        }
    } else {
        (*Message).Response = UserpMessageBox(
            core::ptr::addr_of_mut!(TextU),
            core::ptr::addr_of_mut!(CaptionU),
            dwType,
            Timeout,
        );
    }

    if TextU.Buffer != LocalTextBuffer.as_mut_ptr() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(TextU));
    }
    if CaptionU.Buffer != LocalCaptionBuffer.as_mut_ptr() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(CaptionU));
    }

    NtUserSetInformationThread(
        (-2isize) as *mut core::ffi::c_void,
        UserThreadRestoreDesktop,
        core::ptr::null_mut(),
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn UserInitHardErrorsCache() {
    let Status: NTSTATUS;
    let mut CurrentUserLCID: LCID = 0;

    Status = NtQueryDefaultLocale(1u8, core::ptr::addr_of_mut!(CurrentUserLCID));
    if !NT_SUCCESS(Status) || CurrentUserLCID == 0 {
        CurrentUserLCID = MAKELCID(MAKELANGID(LANG_ENGLISH as u16, SUBLANG_ENGLISH_US as u16), SORT_DEFAULT as u16);
    }
    if g_CurrentUserLangId == LANGIDFROMLCID(CurrentUserLCID) {
        return;
    }

    RtlLoadUnicodeString(
        UserServerDllInstance,
        IDS_SEVERITY_SUCCESS,
        core::ptr::addr_of_mut!(g_SuccessU),
        w!("Success").as_ptr() as PCWSTR,
    );
    RtlLoadUnicodeString(
        UserServerDllInstance,
        IDS_SEVERITY_INFORMATIONAL,
        core::ptr::addr_of_mut!(g_InformationU),
        w!("System Information").as_ptr() as PCWSTR,
    );
    RtlLoadUnicodeString(
        UserServerDllInstance,
        IDS_SEVERITY_WARNING,
        core::ptr::addr_of_mut!(g_WarningU),
        w!("System Warning").as_ptr() as PCWSTR,
    );
    RtlLoadUnicodeString(
        UserServerDllInstance,
        IDS_SEVERITY_ERROR,
        core::ptr::addr_of_mut!(g_ErrorU),
        w!("System Error").as_ptr() as PCWSTR,
    );
    RtlLoadUnicodeString(
        UserServerDllInstance,
        IDS_SYSTEM_PROCESS,
        core::ptr::addr_of_mut!(g_SystemProcessU),
        w!("System Process").as_ptr() as PCWSTR,
    );
    RtlLoadUnicodeString(
        UserServerDllInstance,
        IDS_OK_TERMINATE_PROGRAM,
        core::ptr::addr_of_mut!(g_OKTerminateU),
        w!("Click on OK to terminate the program.").as_ptr() as PCWSTR,
    );
    RtlLoadUnicodeString(
        UserServerDllInstance,
        IDS_CANCEL_DEBUG_PROGRAM,
        core::ptr::addr_of_mut!(g_CancelDebugU),
        w!("Click on CANCEL to debug the program.").as_ptr() as PCWSTR,
    );

    g_CurrentUserLangId = LANGIDFROMLCID(CurrentUserLCID);
}
