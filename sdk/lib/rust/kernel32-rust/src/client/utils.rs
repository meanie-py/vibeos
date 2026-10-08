use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::k32_globals::BasepExeLdrEntry;
use crate::k32_externs::STATUS_INVALID_IMAGE_FORMAT;
use core::ffi::c_void;

// BaseNamedObjectDirectory is owned by another C TU (basesrv/dllmain).
extern "C" {
    static mut BaseNamedObjectDirectory: HANDLE;
}

// ntdll Rtl*Size macros — not exports; declare them.
extern "system" {
    fn RtlUnicodeStringToAnsiSize(UnicodeString: PUNICODE_STRING) -> ULONG;
    fn RtlAnsiStringToUnicodeSize(AnsiString: PANSI_STRING) -> ULONG;
    fn RtlUnicodeStringToOemSize(UnicodeString: PCUNICODE_STRING) -> ULONG;
    fn RtlOemStringToUnicodeSize(OemString: PANSI_STRING) -> ULONG;
}

// ROUND_UP(n, align) == (((n)+(align)-1) & ~((align)-1))
#[inline]
fn ROUND_UP(n: SIZE_T, align: SIZE_T) -> SIZE_T {
    (n.wrapping_add(align).wrapping_sub(1)) & !(align.wrapping_sub(1))
}

// Function-pointer typedefs (NTAPI). These were forked per arch because the bindgen Rtl* fn
// items used to spell the literal per-arch convention, and Rust compares fn-pointer ABIs
// nominally — `system` unified with neither. The islands are normalized now, so the whole
// NTAPI surface is `system` on every arch and so are the file's own Basep*Size fns below.
pub type PRTL_CONVERT_STRING =
    Option<unsafe extern "system" fn(PUNICODE_STRING, PCANSI_STRING, BOOLEAN) -> NTSTATUS>;
pub type PRTL_CONVERT_STRINGA =
    Option<unsafe extern "system" fn(PANSI_STRING, PCUNICODE_STRING, BOOLEAN) -> NTSTATUS>;
pub type PRTL_COUNT_STRING = Option<unsafe extern "system" fn(PUNICODE_STRING) -> ULONG>;
pub type PRTL_COUNT_STRINGA = Option<unsafe extern "system" fn(PANSI_STRING) -> ULONG>;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut Restricted: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
};

// Cross-file referenced by active/translated callers (create.rs) → keep C symbol.
#[no_mangle]
pub static mut bIsFileApiAnsi: BOOL = 1; // TRUE

// Cross-file referenced by active C (dllmain.c, find.c) → keep the C symbol.
#[no_mangle]
pub static mut Basep8BitStringToUnicodeString: PRTL_CONVERT_STRING =
    Some(RtlAnsiStringToUnicodeString);

#[no_mangle]
pub static mut BasepUnicodeStringTo8BitString: PRTL_CONVERT_STRINGA =
    Some(RtlUnicodeStringToAnsiString);

#[no_mangle]
pub static mut BasepUnicodeStringTo8BitSize: PRTL_COUNT_STRING =
    Some(BasepUnicodeStringToAnsiSize);

pub static mut Basep8BitStringToUnicodeSize: PRTL_COUNT_STRINGA =
    Some(BasepAnsiStringToUnicodeSize);
// NTAPI ABI spelled to match the PRTL_COUNT_STRING* typedefs above so these coerce into the
// fn-pointer statics without a cross-ABI cast. That used to mean the arch's literal
// convention, which an ABI string cannot take from a macro parameter — hence the macro and
// its cfg'd pair of invocations. The typedefs are `system` now, so there is one spelling and
// the macro has nothing left to vary.
#[no_mangle]
pub unsafe extern "system" fn BasepUnicodeStringToOemSize(String: PUNICODE_STRING) -> ULONG {
    return RtlUnicodeStringToOemSize(String as PCUNICODE_STRING);
}
#[no_mangle]
pub unsafe extern "system" fn BasepOemStringToUnicodeSize(String: PANSI_STRING) -> ULONG {
    RtlOemStringToUnicodeSize(String)
}
#[no_mangle]
pub unsafe extern "system" fn BasepUnicodeStringToAnsiSize(String: PUNICODE_STRING) -> ULONG {
    RtlUnicodeStringToAnsiSize(String)
}
#[no_mangle]
pub unsafe extern "system" fn BasepAnsiStringToUnicodeSize(String: PANSI_STRING) -> u32 {
    RtlAnsiStringToUnicodeSize(String)
}

#[no_mangle]
pub unsafe extern "system" fn BaseGetNamedObjectDirectory() -> HANDLE {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut DirHandle: HANDLE = core::ptr::null_mut();
    let mut BnoHandle: HANDLE = core::ptr::null_mut();
    let mut Token: HANDLE = core::ptr::null_mut();
    let mut NewToken: HANDLE = core::ptr::null_mut();

    if !BaseNamedObjectDirectory.is_null() {
        return BaseNamedObjectDirectory;
    }

    if (*NtCurrentTeb()).IsImpersonating != 0 {
        Status = NtOpenThreadToken(
            NtCurrentThread(),
            TOKEN_IMPERSONATE,
            1, // TRUE
            &mut Token,
        );
        if !NT_SUCCESS(Status) {
            return BaseNamedObjectDirectory;
        }

        NewToken = core::ptr::null_mut();
        Status = NtSetInformationThread(
            NtCurrentThread(),
            ThreadImpersonationToken,
            &mut NewToken as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<HANDLE>() as u32,
        );
        if !NT_SUCCESS(Status) {
            NtClose(Token);
            return BaseNamedObjectDirectory;
        }
    } else {
        Token = core::ptr::null_mut();
    }

    RtlAcquirePebLock();
    if BaseNamedObjectDirectory.is_null() {
        // InitializeObjectAttributes inline
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = &raw mut (*BaseStaticServerData).NamedObjectDirectory;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenDirectoryObject(
            &mut BnoHandle,
            DIRECTORY_QUERY | DIRECTORY_TRAVERSE | DIRECTORY_CREATE_OBJECT | DIRECTORY_CREATE_SUBDIRECTORY,
            &mut ObjectAttributes,
        );
        if !NT_SUCCESS(Status) {
            Status = NtOpenDirectoryObject(
                &mut DirHandle,
                DIRECTORY_TRAVERSE,
                &mut ObjectAttributes,
            );
            if NT_SUCCESS(Status) {
                // InitializeObjectAttributes inline
                ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                ObjectAttributes.RootDirectory = DirHandle;
                ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
                ObjectAttributes.ObjectName = &raw mut Restricted;
                ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                Status = NtOpenDirectoryObject(
                    &mut BnoHandle,
                    DIRECTORY_QUERY | DIRECTORY_TRAVERSE | DIRECTORY_CREATE_OBJECT | DIRECTORY_CREATE_SUBDIRECTORY,
                    &mut ObjectAttributes,
                );
                NtClose(DirHandle);
            }
        }

        if NT_SUCCESS(Status) {
            BaseNamedObjectDirectory = BnoHandle;
        }
    }

    RtlReleasePebLock();

    if !Token.is_null() {
        NtSetInformationThread(
            NtCurrentThread(),
            ThreadImpersonationToken,
            &mut Token as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<HANDLE>() as u32,
        );
        NtClose(Token);
    }

    BaseNamedObjectDirectory
}

#[no_mangle]
pub unsafe extern "system" fn BasepLocateExeLdrEntry(
    Entry: PLDR_DATA_TABLE_ENTRY,
    Context: PVOID,
    StopEnumeration: PBOOLEAN,
) {
    // ASSERT(Entry)
    if Entry.is_null() {
        RtlAssert(
            b"Entry\0".as_ptr() as PVOID,
            b"dll/win32/kernel32/client/utils.c\0".as_ptr() as PVOID,
            161u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    // ASSERT(Context)
    if Context.is_null() {
        RtlAssert(
            b"Context\0".as_ptr() as PVOID,
            b"dll/win32/kernel32/client/utils.c\0".as_ptr() as PVOID,
            162u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    // ASSERT(StopEnumeration)
    if StopEnumeration.is_null() {
        RtlAssert(
            b"StopEnumeration\0".as_ptr() as PVOID,
            b"dll/win32/kernel32/client/utils.c\0".as_ptr() as PVOID,
            163u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // If entry is already found - signal to stop
    if !BasepExeLdrEntry.is_null() {
        *StopEnumeration = 1u8; // TRUE
        return;
    }

    // Otherwise keep enumerating until we find a match
    if (*Entry).DllBase == Context {
        // It matches, so remember the ldr entry
        BasepExeLdrEntry = Entry;

        // And stop enumeration
        *StopEnumeration = 1u8; // TRUE
    }
}

#[no_mangle]
pub unsafe extern "system" fn Basep8BitStringToStaticUnicodeString(
    String: LPCSTR,
) -> PUNICODE_STRING {
    let StaticString: PUNICODE_STRING = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = RtlInitAnsiStringEx(&raw mut AnsiString, String);
    if !NT_SUCCESS(Status) {
        Status = STATUS_BUFFER_OVERFLOW;
    } else {
        Status = (Basep8BitStringToUnicodeString.unwrap())(StaticString, &raw mut AnsiString, 0);
    }

    if NT_SUCCESS(Status) {
        return StaticString;
    }

    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE);
    } else {
        BaseSetLastNTError(Status);
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn Basep8BitStringToDynamicUnicodeString(
    UnicodeString: PUNICODE_STRING,
    String: LPCSTR,
) -> BOOLEAN {
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = RtlInitAnsiStringEx(&mut AnsiString, String);
    if !NT_SUCCESS(Status) {
        Status = STATUS_BUFFER_OVERFLOW;
    } else {
        Status = (Basep8BitStringToUnicodeString.unwrap())(UnicodeString, &mut AnsiString, 1);
    }

    if NT_SUCCESS(Status) {
        return 1;
    }

    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE);
    } else {
        BaseSetLastNTError(Status);
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn BasepAnsiStringToHeapUnicodeString(
    AnsiString: LPCSTR,
    UnicodeString: *mut LPWSTR,
) {
    let mut AnsiTemp: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeTemp: _UNICODE_STRING = core::mem::zeroed();

    RtlInitAnsiString(&mut AnsiTemp, AnsiString);

    if NT_SUCCESS(RtlAnsiStringToUnicodeString(
        &mut UnicodeTemp,
        &mut AnsiTemp,
        1, // TRUE
    )) {
        *UnicodeString = UnicodeTemp.Buffer;
    } else {
        *UnicodeString = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn BaseFormatTimeOut(
    Timeout: PLARGE_INTEGER,
    dwMilliseconds: DWORD,
) -> PLARGE_INTEGER {
    if dwMilliseconds == INFINITE {
        return core::ptr::null_mut();
    }
    (*Timeout).QuadPart = (dwMilliseconds as i64) * -10000i64;
    Timeout
}

#[no_mangle]
pub unsafe extern "system" fn BaseFormatObjectAttributes(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    SecurityAttributes: PSECURITY_ATTRIBUTES,
    ObjectName: PUNICODE_STRING,
) -> POBJECT_ATTRIBUTES {
    let mut Attributes: ULONG = 0;
    let mut RootDirectory: HANDLE = core::ptr::null_mut();
    let mut SecurityDescriptor: PVOID = core::ptr::null_mut();

    if !SecurityAttributes.is_null() {
        Attributes = if (*SecurityAttributes).bInheritHandle != 0 {
            OBJ_INHERIT
        } else {
            0
        } as ULONG;
        SecurityDescriptor = (*SecurityAttributes).lpSecurityDescriptor as PVOID;
    } else {
        if ObjectName.is_null() {
            return core::ptr::null_mut();
        }
        Attributes = 0;
        SecurityDescriptor = core::ptr::null_mut();
    }

    if !ObjectName.is_null() {
        Attributes |= OBJ_OPENIF;
        RootDirectory = BaseGetNamedObjectDirectory();
    } else {
        RootDirectory = core::ptr::null_mut();
    }

    (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*ObjectAttributes).RootDirectory = RootDirectory;
    (*ObjectAttributes).Attributes = Attributes;
    (*ObjectAttributes).ObjectName = ObjectName;
    (*ObjectAttributes).SecurityDescriptor = SecurityDescriptor;
    (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();

    ObjectAttributes
}

#[no_mangle]
pub unsafe extern "system" fn BaseCreateStack(
    hProcess: HANDLE,
    StackCommit: SIZE_T,
    StackReserve: SIZE_T,
    InitialTeb: PINITIAL_TEB,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Headers: PIMAGE_NT_HEADERS;
    let mut Stack: ULONG_PTR;
    let mut UseGuard: BOOLEAN;
    let PageSize: ULONG;
    let AllocationGranularity: ULONG;
    let mut Dummy: ULONG = 0;
    let mut MinimumStackCommit: SIZE_T;
    let mut GuardPageSize: SIZE_T;

    /* Read page size */
    PageSize = (*BaseStaticServerData).SysInfo.PageSize;
    AllocationGranularity = (*BaseStaticServerData).SysInfo.AllocationGranularity;

    /* Get the Image Headers */
    Headers = RtlImageNtHeader((*(*NtCurrentTeb()).ProcessEnvironmentBlock).ImageBaseAddress);
    if Headers.is_null() {
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    let mut StackReserve = StackReserve;
    let mut StackCommit = StackCommit;

    if StackReserve == 0 {
        StackReserve = (*Headers).OptionalHeader.SizeOfStackReserve as SIZE_T;
    }

    if StackCommit == 0 {
        StackCommit = (*Headers).OptionalHeader.SizeOfStackCommit as SIZE_T;
    } else if StackCommit >= StackReserve {
        /* Grow the reserve beyond the commit, up to 1MB alignment */
        StackReserve = ROUND_UP(StackCommit, 1024 * 1024);
    }

    /* Align everything to Page Size */
    StackCommit = ROUND_UP(StackCommit, PageSize as SIZE_T);
    StackReserve = ROUND_UP(StackReserve, AllocationGranularity as SIZE_T);

    MinimumStackCommit = (*(*NtCurrentTeb()).ProcessEnvironmentBlock).MinimumStackCommit as SIZE_T;
    if (MinimumStackCommit != 0) && (StackCommit < MinimumStackCommit) {
        StackCommit = MinimumStackCommit;
    }

    /* Check if the commit is higher than the reserve */
    if StackCommit >= StackReserve {
        /* Grow the reserve beyond the commit, up to 1MB alignment */
        StackReserve = ROUND_UP(StackCommit, 1024 * 1024);
    }

    /* Align everything to Page Size */
    StackCommit = ROUND_UP(StackCommit, PageSize as SIZE_T);
    StackReserve = ROUND_UP(StackReserve, AllocationGranularity as SIZE_T);

    /* Reserve memory for the stack */
    Stack = 0;
    Status = NtAllocateVirtualMemory(
        hProcess,
        &raw mut Stack as *mut PVOID,
        0,
        &raw mut StackReserve,
        MEM_RESERVE,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Now set up some basic Initial TEB Parameters */
    (*InitialTeb).AllocatedStackBase = Stack as PVOID;
    (*InitialTeb).StackBase = (Stack as SIZE_T).wrapping_add(StackReserve) as PVOID;
    (*InitialTeb).PreviousStackBase = core::ptr::null_mut();
    (*InitialTeb).PreviousStackLimit = core::ptr::null_mut();

    /* Update the stack position */
    Stack = (Stack as SIZE_T).wrapping_add(StackReserve.wrapping_sub(StackCommit)) as ULONG_PTR;

    /* Check if we can add a guard page */
    if StackReserve >= (StackCommit as SIZE_T).wrapping_add(PageSize as SIZE_T) {
        Stack = (Stack as SIZE_T).wrapping_sub(PageSize as SIZE_T) as ULONG_PTR;
        StackCommit = (StackCommit as SIZE_T).wrapping_add(PageSize as SIZE_T) as SIZE_T;
        UseGuard = 1;
    } else {
        UseGuard = 0;
    }

    /* Allocate memory for the stack */
    Status = NtAllocateVirtualMemory(
        hProcess,
        &raw mut Stack as *mut PVOID,
        0,
        &raw mut StackCommit,
        MEM_COMMIT,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(Status) {
        GuardPageSize = 0;
        NtFreeVirtualMemory(
            hProcess,
            &raw mut Stack as *mut PVOID,
            &raw mut GuardPageSize,
            MEM_RELEASE,
        );
        return Status;
    }

    /* Now set the current Stack Limit */
    (*InitialTeb).StackLimit = Stack as PVOID;

    /* Create a guard page if needed */
    if UseGuard != 0 {
        GuardPageSize = PageSize as SIZE_T;
        Status = NtProtectVirtualMemory(
            hProcess,
            &raw mut Stack as *mut PVOID,
            &raw mut GuardPageSize,
            PAGE_GUARD | PAGE_READWRITE,
            &raw mut Dummy,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        /* Update the Stack Limit keeping in mind the Guard Page */
        (*InitialTeb).StackLimit = ((*InitialTeb).StackLimit as SIZE_T)
            .wrapping_add(GuardPageSize) as PVOID;
    }

    /* We are done! */
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn BaseFreeThreadStack(
    hProcess: HANDLE,
    InitialTeb: PINITIAL_TEB,
) {
    let mut Dummy: SIZE_T = 0;

    NtFreeVirtualMemory(
        hProcess,
        core::ptr::addr_of_mut!((*InitialTeb).AllocatedStackBase),
        core::ptr::addr_of_mut!(Dummy),
        MEM_RELEASE,
    );
}

// i386 asm thunks are PUBLIC _BaseThreadStartupThunk@0 etc. (stdcall-decorated,
// @0). `extern "system"` on a zero-arg fn produces the `@0` decoration to match.
#[cfg(target_arch = "x86")]
extern "system" {
    fn BaseThreadStartupThunk();
    fn BaseProcessStartThunk();
    fn BaseFiberStartup();
}

#[cfg(target_arch = "x86_64")]
extern "C" {
    fn BaseThreadStartup();
    fn BaseFiberStartup();
    fn BaseProcessStartup();
}

// arm64: BaseThreadStartup (thread.rs) / BaseProcessStartup (proc_helpers.rs) /
// BaseFiberStartup (client/arm64/fiber.S). extern "C"==extern "system" here.
#[cfg(target_arch = "aarch64")]
extern "C" {
    fn BaseThreadStartup();
    fn BaseFiberStartup();
    fn BaseProcessStartup();
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn BaseInitializeContext(
    Context: PCONTEXT,
    Parameter: PVOID,
    StartAddress: PVOID,
    StackAddress: PVOID,
    ContextType: ULONG,
) {
    const KGDT_R3_CODE: WORD = 0x18;
    const KGDT_R3_DATA: WORD = 0x20;
    const KGDT_R3_TEB: WORD = 0x38;

    let ContextFlags: ULONG;

    /* Setup the Initial Win32 Thread Context */
    (*Context).Eax = StartAddress as ULONG_PTR as DWORD;
    (*Context).Ebx = Parameter as ULONG_PTR as DWORD;
    (*Context).Esp = StackAddress as ULONG_PTR as DWORD;
    /* The other registers are undefined */

    /* Setup the Segments */
    (*Context).SegFs = KGDT_R3_TEB as DWORD;
    (*Context).SegEs = KGDT_R3_DATA as DWORD;
    (*Context).SegDs = KGDT_R3_DATA as DWORD;
    (*Context).SegCs = KGDT_R3_CODE as DWORD;
    (*Context).SegSs = KGDT_R3_DATA as DWORD;
    (*Context).SegGs = 0;

    /* Set the Context Flags */
    ContextFlags = (*Context).ContextFlags;
    (*Context).ContextFlags = CONTEXT_FULL as DWORD;

    /* Give it some room for the Parameter */
    (*Context).Esp = (*Context).Esp
        .wrapping_sub(core::mem::size_of::<PVOID>() as DWORD);

    /* Set the EFLAGS */
    (*Context).EFlags = 0x3000; /* IOPL 3 */

    /* What kind of context is being created? */
    if ContextType == 1 {
        /* For Threads */
        (*Context).Eip = BaseThreadStartupThunk as ULONG_PTR as DWORD;
    } else if ContextType == 2 {
        /* This is a fiber: make space for the return address */
        (*Context).Esp = (*Context).Esp
            .wrapping_sub(core::mem::size_of::<PVOID>() as DWORD);
        *((*Context).Esp as *mut PVOID) = BaseFiberStartup as PVOID;

        /* Is FPU state required? */
        (*Context).ContextFlags |= ContextFlags;
        if (ContextFlags & CONTEXT_FLOATING_POINT) == CONTEXT_FLOATING_POINT {
            /* Set an initial state */
            (*Context).FloatSave.ControlWord = 0x27F;
            (*Context).FloatSave.StatusWord = 0;
            (*Context).FloatSave.TagWord = 0xFFFF;
            (*Context).FloatSave.ErrorOffset = 0;
            (*Context).FloatSave.ErrorSelector = 0;
            (*Context).FloatSave.DataOffset = 0;
            (*Context).FloatSave.DataSelector = 0;
            if (*SharedUserData).ProcessorFeatures
                [PF_XMMI_INSTRUCTIONS_AVAILABLE as usize]
                != 0
            {
                (*Context).Dr6 = 0x1F80;
            }
        }
    } else {
        /* For first thread in a Process */
        (*Context).Eip = BaseProcessStartThunk as ULONG_PTR as DWORD;
    }
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn BaseInitializeContext(
    Context: PCONTEXT,
    Parameter: PVOID,
    StartAddress: PVOID,
    StackAddress: PVOID,
    ContextType: ULONG,
) {
    const RPL_MASK: WORD = 0x3;
    const KGDT64_R3_DATA: WORD = 0x28;
    const KGDT64_R3_CODE: WORD = 0x30;
    const KGDT64_R3_CMTEB: WORD = 0x50;
    const EFLAGS_INTERRUPT_MASK: u32 = 0x200;
    const INITIAL_MXCSR: u32 = 0x1f80;

    // ASSERT(((ULONG_PTR)StackAddress & 15) == 0);
    if (StackAddress as usize) & 15 != 0 {
        DbgBreakPoint();
    }

    core::ptr::write_bytes(Context as *mut u8, 0, core::mem::size_of::<CONTEXT>());

    /* Setup the Initial Win32 Thread Context */
    (*Context).Rcx = StartAddress as ULONG_PTR as DWORD64;
    (*Context).Rdx = Parameter as ULONG_PTR as DWORD64;
    (*Context).Rsp = (StackAddress as usize)
        .wrapping_sub(5usize.wrapping_mul(core::mem::size_of::<PVOID>()))
        as DWORD64;

    /* Setup the Segments */
    (*Context).SegGs = KGDT64_R3_DATA | RPL_MASK;
    (*Context).SegEs = KGDT64_R3_DATA | RPL_MASK;
    (*Context).SegDs = KGDT64_R3_DATA | RPL_MASK;
    (*Context).SegCs = KGDT64_R3_CODE | RPL_MASK;
    (*Context).SegSs = KGDT64_R3_DATA | RPL_MASK;
    (*Context).SegFs = KGDT64_R3_CMTEB | RPL_MASK;

    /* Set the EFLAGS */
    (*Context).EFlags = (0x3000u32 | EFLAGS_INTERRUPT_MASK) as DWORD;

    /* Set MXCSR */
    (*Context).MxCsr = INITIAL_MXCSR as DWORD;

    if ContextType == 1 {
        /* For Threads */
        (*Context).Rip = BaseThreadStartup as ULONG_PTR as DWORD64;
    } else if ContextType == 2 {
        /* For Fibers */
        (*Context).Rip = BaseFiberStartup as ULONG_PTR as DWORD64;
    } else {
        /* For first thread in a Process */
        (*Context).Rip = BaseProcessStartup as ULONG_PTR as DWORD64;
    }

    /* Set the Context Flags */
    (*Context).ContextFlags = CONTEXT_FULL as DWORD;
}

// kernel32 client/utils.c HAS an `_M_ARM64` branch (utils.c:648) — faithfully
// translated here. (An earlier stub left this empty, so the zeroed CONTEXT reached
// NtCreateThread with Pc=0/X0=0 → every thread started at PspUserThreadStartup with
// StartRoutine=0 → BUGCHECK 0xF4 when the thread belonged to a critical process.)
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "system" fn BaseInitializeContext(
    Context: PCONTEXT,
    Parameter: PVOID,
    StartAddress: PVOID,
    StackAddress: PVOID,
    ContextType: ULONG,
) {
    core::ptr::write_bytes(Context as *mut u8, 0, core::mem::size_of::<CONTEXT>());

    /* Setup the Initial Win32 Thread Context (ARM64 calling convention) */
    (*Context).__bindgen_anon_1.__bindgen_anon_1.X0 = StartAddress as ULONG_PTR as DWORD64; /* 1st arg */
    (*Context).__bindgen_anon_1.__bindgen_anon_1.X1 = Parameter as ULONG_PTR as DWORD64;    /* 2nd arg */
    (*Context).Sp = (StackAddress as ULONG_PTR as DWORD64) & !15u64;                        /* 16-byte aligned */

    if ContextType == 1 {
        /* For Threads */
        (*Context).Pc = BaseThreadStartup as ULONG_PTR as DWORD64;
    } else if ContextType == 2 {
        /* For Fibers */
        (*Context).Pc = BaseFiberStartup as ULONG_PTR as DWORD64;
    } else {
        /* For first thread in a Process */
        (*Context).Pc = BaseProcessStartup as ULONG_PTR as DWORD64;
    }

    /* Set the Context Flags */
    (*Context).ContextFlags = CONTEXT_FULL as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn BasepIsRealtimeAllowed(Keep: BOOLEAN) -> PVOID {
    let mut Privilege: ULONG = SE_INC_BASE_PRIORITY_PRIVILEGE as ULONG;
    let mut State: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    Status = RtlAcquirePrivilege(&raw mut Privilege, 1 as ULONG, 0 as ULONG, &raw mut State);
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }
    if Keep == 0 {
        RtlReleasePrivilege(State);
        State = 1 as PVOID;
    }
    return State;
}

#[no_mangle]
pub unsafe extern "system" fn BasepMapFile(
    lpApplicationName: LPCWSTR,
    hSection: PHANDLE,
    ApplicationName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut ApplicationName = ApplicationName;
    let mut RelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    RelativeName.ContainingDirectory = core::ptr::null_mut();

    if RtlDosPathNameToNtPathName_U(
        lpApplicationName,
        ApplicationName,
        core::ptr::null_mut(),
        &mut RelativeName,
    ) == 0
    {
        return STATUS_OBJECT_PATH_NOT_FOUND;
    }

    if RelativeName.RelativeName.Length != 0 {
        ApplicationName = &mut RelativeName.RelativeName;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = RelativeName.ContainingDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = ApplicationName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut hFile,
        (SYNCHRONIZE | FILE_EXECUTE | FILE_READ_DATA) as u32,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        (FILE_SHARE_DELETE | FILE_SHARE_READ) as u32,
        (FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE) as u32,
    );
    // The relative-name handle RtlDosPathNameToNtPathName_U opened must be released
    // whatever NtOpenFile returned (upstream ef8bcd3fbe7; it leaked once per call before).
    RtlReleaseRelativeName(&mut RelativeName);

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return Status;
    }

    Status = NtCreateSection(
        hSection,
        SECTION_ALL_ACCESS as u32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        PAGE_EXECUTE as u32,
        SEC_IMAGE as u32,
        hFile,
    );

    NtClose(hFile);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn Wow64EnableWow64FsRedirection(
    Wow64EnableWow64FsRedirection: BOOLEAN,
) -> BOOLEAN {
    let Status: NTSTATUS;
    let Result: BOOLEAN;

    Status = RtlWow64EnableFsRedirection(Wow64EnableWow64FsRedirection);
    if NT_SUCCESS(Status) {
        Result = 1;
    } else {
        BaseSetLastNTError(Status);
        Result = 0;
    }
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn Wow64DisableWow64FsRedirection(
    OldValue: *mut *mut core::ffi::c_void,
) -> BOOL {
    let Status: NTSTATUS;
    let Result: BOOL;

    Status = RtlWow64EnableFsRedirectionEx(1 as PVOID, OldValue);
    if NT_SUCCESS(Status) {
        Result = 1;
    } else {
        BaseSetLastNTError(Status);
        Result = 0;
    }
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn Wow64RevertWow64FsRedirection(mut OldValue: PVOID) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut Result: BOOL = 0;

    Status = RtlWow64EnableFsRedirectionEx(OldValue, &mut OldValue);
    if NT_SUCCESS(Status) {
        Result = 1;
    } else {
        BaseSetLastNTError(Status);
        Result = 0;
    }
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn SetFileApisToOEM() {
    /* Set the correct Base Api */
    // C: Basep8BitStringToUnicodeString = (PRTL_CONVERT_STRING)RtlOemStringToUnicodeString;
    // RtlOemStringToUnicodeString's SourceString is PCOEM_STRING (*const STRING), while
    // PRTL_CONVERT_STRING expects PCANSI_STRING (*mut STRING); the C cast erases that, so
    // transmute the differing fn-pointer type here.
    type OemConvert =
        Option<unsafe extern "system" fn(PUNICODE_STRING, PCOEM_STRING, BOOLEAN) -> NTSTATUS>;
    Basep8BitStringToUnicodeString =
        ::core::mem::transmute::<OemConvert, PRTL_CONVERT_STRING>(Some(RtlOemStringToUnicodeString));
    // RtlUnicodeStringToOemString: (POEM_STRING, PCUNICODE_STRING, BOOLEAN); POEM_STRING ==
    // PANSI_STRING == PSTRING, so this coerces to PRTL_CONVERT_STRINGA directly.
    BasepUnicodeStringTo8BitString = Some(RtlUnicodeStringToOemString);
    BasepUnicodeStringTo8BitSize = Some(BasepUnicodeStringToOemSize);
    Basep8BitStringToUnicodeSize = Some(BasepOemStringToUnicodeSize);

    /* FIXME: Old, deprecated way */
    bIsFileApiAnsi = 0; // FALSE
}

#[no_mangle]
pub unsafe extern "system" fn SetFileApisToANSI() {
    /* Set the correct Base Api */
    Basep8BitStringToUnicodeString = Some(RtlAnsiStringToUnicodeString);
    BasepUnicodeStringTo8BitString = Some(RtlUnicodeStringToAnsiString);
    BasepUnicodeStringTo8BitSize = Some(BasepUnicodeStringToAnsiSize);
    Basep8BitStringToUnicodeSize = Some(BasepAnsiStringToUnicodeSize);

    /* FIXME: Old, deprecated way */
    bIsFileApiAnsi = 1; // TRUE
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn AreFileApisANSI() -> BOOL {
    (Basep8BitStringToUnicodeString == Some(RtlAnsiStringToUnicodeString)) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn BaseMarkFileForDelete(
    FileHandle: HANDLE,
    FileAttributes: ULONG,
) {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileBasicInfoData: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut FileDispositionInfoData: FILE_DISPOSITION_INFORMATION = core::mem::zeroed();
    let mut FileAttributes = FileAttributes;

    if FileAttributes == 0 {
        FileBasicInfoData.FileAttributes = 0;
        NtQueryInformationFile(
            FileHandle,
            &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
            &mut FileBasicInfoData as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
            FileBasicInformation,
        );
        FileAttributes = FileBasicInfoData.FileAttributes;
    }

    if FileAttributes & FILE_ATTRIBUTE_READONLY != 0 {
        core::ptr::write_bytes(
            &mut FileBasicInfoData as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<FILE_BASIC_INFORMATION>(),
        );
        FileBasicInfoData.FileAttributes = FILE_ATTRIBUTE_NORMAL;
        NtSetInformationFile(
            FileHandle,
            &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
            &mut FileBasicInfoData as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
            FileBasicInformation,
        );
    }

    FileDispositionInfoData.DeleteFileA = 1u8; // TRUE
    NtSetInformationFile(
        FileHandle,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FileDispositionInfoData as *mut FILE_DISPOSITION_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_DISPOSITION_INFORMATION>() as ULONG,
        FileDispositionInformation,
    );
}

#[no_mangle]
pub unsafe extern "system" fn BasepCheckWinSaferRestrictions(
    UserToken: HANDLE,
    ApplicationName: LPWSTR,
    FileHandle: HANDLE,
    InJob: PBOOLEAN,
    NewToken: PHANDLE,
    JobHandle: PHANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;

    /* Validate that there's a name */
    if !ApplicationName.is_null() && *ApplicationName != 0 {
        /* Validate that the required output parameters are there */
        if !InJob.is_null() && !NewToken.is_null() && !JobHandle.is_null() {
            /* Do the work (one day...) */
            Status = STATUS_SUCCESS;
        } else {
            /* Act as if SEH hit this */
            Status = STATUS_ACCESS_VIOLATION;
        }
    } else {
        /* Input is invalid */
        Status = STATUS_INVALID_PARAMETER;
    }

    /* Return the status */
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/utils.c
//  * PURPOSE:         Utility and Support Functions
//  * PROGRAMMER:      Alex Ionescu (alex@relsoft.net)
//  *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <k32.h>
// #ifdef _M_IX86
// #include "i386/ketypes.h"
// #elif defined _M_AMD64
// #include "amd64/ketypes.h"
// #endif
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// UNICODE_STRING Restricted = RTL_CONSTANT_STRING(L"Restricted");
// BOOL bIsFileApiAnsi = TRUE; // set the file api to ansi or oem
// PRTL_CONVERT_STRING Basep8BitStringToUnicodeString = RtlAnsiStringToUnicodeString;
// PRTL_CONVERT_STRINGA BasepUnicodeStringTo8BitString = RtlUnicodeStringToAnsiString;
// PRTL_COUNT_STRING BasepUnicodeStringTo8BitSize = BasepUnicodeStringToAnsiSize;
// PRTL_COUNT_STRINGA Basep8BitStringToUnicodeSize = BasepAnsiStringToUnicodeSize;
//
// /* FUNCTIONS ******************************************************************/
//
// ULONG
// NTAPI
// BasepUnicodeStringToOemSize(IN PUNICODE_STRING String)
// {
//     return RtlUnicodeStringToOemSize(String);
// }
//
// ULONG
// NTAPI
// BasepOemStringToUnicodeSize(IN PANSI_STRING String)
// {
//     return RtlOemStringToUnicodeSize(String);
// }
//
// ULONG
// NTAPI
// BasepUnicodeStringToAnsiSize(IN PUNICODE_STRING String)
// {
//     return RtlUnicodeStringToAnsiSize(String);
// }
//
// ULONG
// NTAPI
// BasepAnsiStringToUnicodeSize(IN PANSI_STRING String)
// {
//     return RtlAnsiStringToUnicodeSize(String);
// }
//
// HANDLE
// WINAPI
// BaseGetNamedObjectDirectory(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE DirHandle, BnoHandle, Token, NewToken;
//
//     if (BaseNamedObjectDirectory) return BaseNamedObjectDirectory;
//
//     if (NtCurrentTeb()->IsImpersonating)
//     {
//         Status = NtOpenThreadToken(NtCurrentThread(),
//                                    TOKEN_IMPERSONATE,
//                                    TRUE,
//                                    &Token);
//         if (!NT_SUCCESS(Status)) return BaseNamedObjectDirectory;
//
//         NewToken = NULL;
//         Status = NtSetInformationThread(NtCurrentThread(),
//                                         ThreadImpersonationToken,
//                                         &NewToken,
//                                         sizeof(HANDLE));
//         if (!NT_SUCCESS (Status))
//         {
//             NtClose(Token);
//             return BaseNamedObjectDirectory;
//         }
//     }
//     else
//     {
//         Token = NULL;
//     }
//
//     RtlAcquirePebLock();
//     if (BaseNamedObjectDirectory) goto Quickie;
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &BaseStaticServerData->NamedObjectDirectory,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenDirectoryObject(&BnoHandle,
//                                    DIRECTORY_QUERY |
//                                    DIRECTORY_TRAVERSE |
//                                    DIRECTORY_CREATE_OBJECT |
//                                    DIRECTORY_CREATE_SUBDIRECTORY,
//                                    &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         Status = NtOpenDirectoryObject(&DirHandle,
//                                        DIRECTORY_TRAVERSE,
//                                        &ObjectAttributes);
//
//         if (NT_SUCCESS(Status))
//         {
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        (PUNICODE_STRING)&Restricted,
//                                        OBJ_CASE_INSENSITIVE,
//                                        DirHandle,
//                                        NULL);
//
//             Status = NtOpenDirectoryObject(&BnoHandle,
//                                            DIRECTORY_QUERY |
//                                            DIRECTORY_TRAVERSE |
//                                            DIRECTORY_CREATE_OBJECT |
//                                            DIRECTORY_CREATE_SUBDIRECTORY,
//                                            &ObjectAttributes);
//             NtClose(DirHandle);
//
//         }
//     }
//
//     if (NT_SUCCESS(Status)) BaseNamedObjectDirectory = BnoHandle;
//
// Quickie:
//
//     RtlReleasePebLock();
//
//     if (Token)
//     {
//         NtSetInformationThread(NtCurrentThread(),
//                                ThreadImpersonationToken,
//                                &Token,
//                                sizeof(Token));
//
//         NtClose(Token);
//     }
//
//     return BaseNamedObjectDirectory;
// }
//
// VOID
// NTAPI
// BasepLocateExeLdrEntry(IN PLDR_DATA_TABLE_ENTRY Entry,
//                        IN PVOID Context,
//                        OUT BOOLEAN *StopEnumeration)
// {
//     /* Make sure we get Entry, Context and valid StopEnumeration pointer */
//     ASSERT(Entry);
//     ASSERT(Context);
//     ASSERT(StopEnumeration);
//
//     /* If entry is already found - signal to stop */
//     if (BasepExeLdrEntry)
//     {
//         *StopEnumeration = TRUE;
//         return;
//     }
//
//     /* Otherwise keep enumerating until we find a match */
//     if (Entry->DllBase == Context)
//     {
//         /* It matches, so remember the ldr entry */
//         BasepExeLdrEntry = Entry;
//
//         /* And stop enumeration */
//         *StopEnumeration = TRUE;
//     }
// }
//
// /*
//  * Converts an ANSI or OEM String to the TEB StaticUnicodeString
//  */
// PUNICODE_STRING
// WINAPI
// Basep8BitStringToStaticUnicodeString(IN LPCSTR String)
// {
//     PUNICODE_STRING StaticString = &(NtCurrentTeb()->StaticUnicodeString);
//     ANSI_STRING AnsiString;
//     NTSTATUS Status;
//
//     /* Initialize an ANSI String */
//     Status = RtlInitAnsiStringEx(&AnsiString, String);
//     if (!NT_SUCCESS(Status))
//     {
//         Status = STATUS_BUFFER_OVERFLOW;
//     }
//     else
//     {
//         /* Convert it */
//         Status = Basep8BitStringToUnicodeString(StaticString, &AnsiString, FALSE);
//     }
//
//     if (NT_SUCCESS(Status)) return StaticString;
//
//     if (Status == STATUS_BUFFER_OVERFLOW)
//     {
//         SetLastError(ERROR_FILENAME_EXCED_RANGE);
//     }
//     else
//     {
//         BaseSetLastNTError(Status);
//     }
//
//     return NULL;
// }
//
// /*
//  * Allocates space from the Heap and converts an Unicode String into it
//  */
// BOOLEAN
// WINAPI
// Basep8BitStringToDynamicUnicodeString(OUT PUNICODE_STRING UnicodeString,
//                                       IN LPCSTR String)
// {
//     ANSI_STRING AnsiString;
//     NTSTATUS Status;
//
//     /* Initialize an ANSI String */
//     Status = RtlInitAnsiStringEx(&AnsiString, String);
//     if (!NT_SUCCESS(Status))
//     {
//         Status = STATUS_BUFFER_OVERFLOW;
//     }
//     else
//     {
//         /* Convert it */
//         Status = Basep8BitStringToUnicodeString(UnicodeString, &AnsiString, TRUE);
//     }
//
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     if (Status == STATUS_BUFFER_OVERFLOW)
//     {
//         SetLastError(ERROR_FILENAME_EXCED_RANGE);
//     }
//     else
//     {
//         BaseSetLastNTError(Status);
//     }
//
//     return FALSE;
// }
//
// /*
//  * Allocates space from the Heap and converts an Ansi String into it
//  */
//  /*NOTE: API IS A HACK */
// VOID
// WINAPI
// BasepAnsiStringToHeapUnicodeString(IN LPCSTR AnsiString,
//                                    OUT LPWSTR* UnicodeString)
// {
//     ANSI_STRING AnsiTemp;
//     UNICODE_STRING UnicodeTemp;
//
//     DPRINT("BasepAnsiStringToHeapUnicodeString\n");
//
//     /* First create the ANSI_STRING */
//     RtlInitAnsiString(&AnsiTemp, AnsiString);
//
//     if (NT_SUCCESS(RtlAnsiStringToUnicodeString(&UnicodeTemp,
//                                                 &AnsiTemp,
//                                                 TRUE)))
//     {
//         *UnicodeString = UnicodeTemp.Buffer;
//     }
//     else
//     {
//         *UnicodeString = NULL;
//     }
// }
//
// PLARGE_INTEGER
// WINAPI
// BaseFormatTimeOut(OUT PLARGE_INTEGER Timeout,
//                   IN DWORD dwMilliseconds)
// {
//     /* Check if this is an infinite wait, which means no timeout argument */
//     if (dwMilliseconds == INFINITE) return NULL;
//
//     /* Otherwise, convert the time to NT Format */
//     Timeout->QuadPart = dwMilliseconds * -10000LL;
//     return Timeout;
// }
//
// /*
//  * Converts lpSecurityAttributes + Object Name into ObjectAttributes.
//  */
// POBJECT_ATTRIBUTES
// WINAPI
// BaseFormatObjectAttributes(OUT POBJECT_ATTRIBUTES ObjectAttributes,
//                            IN PSECURITY_ATTRIBUTES SecurityAttributes OPTIONAL,
//                            IN PUNICODE_STRING ObjectName)
// {
//     ULONG Attributes;
//     HANDLE RootDirectory;
//     PVOID SecurityDescriptor;
//     DPRINT("BaseFormatObjectAttributes. Security: %p, Name: %p\n",
//             SecurityAttributes, ObjectName);
//
//     /* Get the attributes if present */
//     if (SecurityAttributes)
//     {
//         Attributes = SecurityAttributes->bInheritHandle ? OBJ_INHERIT : 0;
//         SecurityDescriptor = SecurityAttributes->lpSecurityDescriptor;
//     }
//     else
//     {
//         if (!ObjectName) return NULL;
//         Attributes = 0;
//         SecurityDescriptor = NULL;
//     }
//
//     if (ObjectName)
//     {
//         Attributes |= OBJ_OPENIF;
//         RootDirectory = BaseGetNamedObjectDirectory();
//     }
//     else
//     {
//         RootDirectory = NULL;
//     }
//
//     /* Create the Object Attributes */
//     InitializeObjectAttributes(ObjectAttributes,
//                                ObjectName,
//                                Attributes,
//                                RootDirectory,
//                                SecurityDescriptor);
//     DPRINT("Attributes: %lx, RootDirectory: %p, SecurityDescriptor: %p\n",
//             Attributes, RootDirectory, SecurityDescriptor);
//     return ObjectAttributes;
// }
//
// /*
//  * Creates a stack for a thread or fiber.
//  * NOTE: Adapted from sdk/lib/rtl/thread.c:RtlpCreateUserStack().
//  */
// NTSTATUS
// WINAPI
// BaseCreateStack(
//      HANDLE hProcess,
//      SIZE_T StackCommit,
//      SIZE_T StackReserve,
//      PINITIAL_TEB InitialTeb)
// {
//     NTSTATUS Status;
//     PIMAGE_NT_HEADERS Headers;
//     ULONG_PTR Stack;
//     BOOLEAN UseGuard;
//     ULONG PageSize, AllocationGranularity, Dummy;
//     SIZE_T MinimumStackCommit, GuardPageSize;
//
//     DPRINT("BaseCreateStack(hProcess: 0x%p, Max: 0x%lx, Current: 0x%lx)\n",
//             hProcess, StackReserve, StackCommit);
//
//     /* Read page size */
//     PageSize = BaseStaticServerData->SysInfo.PageSize;
//     AllocationGranularity = BaseStaticServerData->SysInfo.AllocationGranularity;
//
//     /* Get the Image Headers */
//     Headers = RtlImageNtHeader(NtCurrentPeb()->ImageBaseAddress);
//     if (!Headers) return STATUS_INVALID_IMAGE_FORMAT;
//
//     if (StackReserve == 0)
//         StackReserve = Headers->OptionalHeader.SizeOfStackReserve;
//
//     if (StackCommit == 0)
//     {
//         StackCommit = Headers->OptionalHeader.SizeOfStackCommit;
//     }
//     /* Check if the commit is higher than the reserve */
//     else if (StackCommit >= StackReserve)
//     {
//         /* Grow the reserve beyond the commit, up to 1MB alignment */
//         StackReserve = ROUND_UP(StackCommit, 1024 * 1024);
//     }
//
//     /* Align everything to Page Size */
//     StackCommit = ROUND_UP(StackCommit, PageSize);
//     StackReserve = ROUND_UP(StackReserve, AllocationGranularity);
//
//     MinimumStackCommit = NtCurrentPeb()->MinimumStackCommit;
//     if ((MinimumStackCommit != 0) && (StackCommit < MinimumStackCommit))
//     {
//         StackCommit = MinimumStackCommit;
//     }
//
//     /* Check if the commit is higher than the reserve */
//     if (StackCommit >= StackReserve)
//     {
//         /* Grow the reserve beyond the commit, up to 1MB alignment */
//         StackReserve = ROUND_UP(StackCommit, 1024 * 1024);
//     }
//
//     /* Align everything to Page Size */
//     StackCommit = ROUND_UP(StackCommit, PageSize);
//     StackReserve = ROUND_UP(StackReserve, AllocationGranularity);
//
//     /* Reserve memory for the stack */
//     Stack = 0;
//     Status = NtAllocateVirtualMemory(hProcess,
//                                      (PVOID*)&Stack,
//                                      0,
//                                      &StackReserve,
//                                      MEM_RESERVE,
//                                      PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failure to reserve stack: %lx\n", Status);
//         return Status;
//     }
//
//     /* Now set up some basic Initial TEB Parameters */
//     InitialTeb->AllocatedStackBase = (PVOID)Stack;
//     InitialTeb->StackBase = (PVOID)(Stack + StackReserve);
//     InitialTeb->PreviousStackBase = NULL;
//     InitialTeb->PreviousStackLimit = NULL;
//
//     /* Update the stack position */
//     Stack += StackReserve - StackCommit;
//
//     /* Check if we can add a guard page */
//     if (StackReserve >= StackCommit + PageSize)
//     {
//         Stack -= PageSize;
//         StackCommit += PageSize;
//         UseGuard = TRUE;
//     }
//     else
//     {
//         UseGuard = FALSE;
//     }
//
//     /* Allocate memory for the stack */
//     Status = NtAllocateVirtualMemory(hProcess,
//                                      (PVOID*)&Stack,
//                                      0,
//                                      &StackCommit,
//                                      MEM_COMMIT,
//                                      PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failure to allocate stack\n");
//         GuardPageSize = 0;
//         NtFreeVirtualMemory(hProcess, (PVOID*)&Stack, &GuardPageSize, MEM_RELEASE);
//         return Status;
//     }
//
//     /* Now set the current Stack Limit */
//     InitialTeb->StackLimit = (PVOID)Stack;
//
//     /* Create a guard page if needed */
//     if (UseGuard)
//     {
//         GuardPageSize = PageSize;
//         Status = NtProtectVirtualMemory(hProcess,
//                                         (PVOID*)&Stack,
//                                         &GuardPageSize,
//                                         PAGE_GUARD | PAGE_READWRITE,
//                                         &Dummy);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failure to set guard page\n");
//             return Status;
//         }
//
//         /* Update the Stack Limit keeping in mind the Guard Page */
//         InitialTeb->StackLimit = (PVOID)((ULONG_PTR)InitialTeb->StackLimit +
//                                          GuardPageSize);
//     }
//
//     /* We are done! */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * NOTE: Adapted from sdk/lib/rtl/thread.c:RtlpFreeUserStack().
//  */
// VOID
// WINAPI
// BaseFreeThreadStack(
//      HANDLE hProcess,
//      PINITIAL_TEB InitialTeb)
// {
//     SIZE_T Dummy = 0;
//
//     /* Free the Stack */
//     NtFreeVirtualMemory(hProcess,
//                         &InitialTeb->AllocatedStackBase,
//                         &Dummy,
//                         MEM_RELEASE);
// }
//
// /*
//  * Creates the Initial Context for a Thread or Fiber
//  */
// VOID
// WINAPI
// BaseInitializeContext(IN PCONTEXT Context,
//                       IN PVOID Parameter,
//                       IN PVOID StartAddress,
//                       IN PVOID StackAddress,
//                       IN ULONG ContextType)
// {
// #ifdef _M_IX86
//     ULONG ContextFlags;
//     DPRINT("BaseInitializeContext: %p\n", Context);
//
//     /* Setup the Initial Win32 Thread Context */
//     Context->Eax = (ULONG)StartAddress;
//     Context->Ebx = (ULONG)Parameter;
//     Context->Esp = (ULONG)StackAddress;
//     /* The other registers are undefined */
//
//     /* Setup the Segments */
//     Context->SegFs = KGDT_R3_TEB;
//     Context->SegEs = KGDT_R3_DATA;
//     Context->SegDs = KGDT_R3_DATA;
//     Context->SegCs = KGDT_R3_CODE;
//     Context->SegSs = KGDT_R3_DATA;
//     Context->SegGs = 0;
//
//     /* Set the Context Flags */
//     ContextFlags = Context->ContextFlags;
//     Context->ContextFlags = CONTEXT_FULL;
//
//     /* Give it some room for the Parameter */
//     Context->Esp -= sizeof(PVOID);
//
//     /* Set the EFLAGS */
//     Context->EFlags = 0x3000; /* IOPL 3 */
//
//     /* What kind of context is being created? */
//     if (ContextType == 1)
//     {
//         /* For Threads */
//         Context->Eip = (ULONG)BaseThreadStartupThunk;
//     }
//     else if (ContextType == 2)
//     {
//         /* This is a fiber: make space for the return address */
//         Context->Esp -= sizeof(PVOID);
//         *((PVOID*)Context->Esp) = BaseFiberStartup;
//
//         /* Is FPU state required? */
//         Context->ContextFlags |= ContextFlags;
//         if ((ContextFlags & CONTEXT_FLOATING_POINT) == CONTEXT_FLOATING_POINT)
//         {
//             /* Set an initial state */
//             Context->FloatSave.ControlWord = 0x27F;
//             Context->FloatSave.StatusWord = 0;
//             Context->FloatSave.TagWord = 0xFFFF;
//             Context->FloatSave.ErrorOffset = 0;
//             Context->FloatSave.ErrorSelector = 0;
//             Context->FloatSave.DataOffset = 0;
//             Context->FloatSave.DataSelector = 0;
//             if (SharedUserData->ProcessorFeatures[PF_XMMI_INSTRUCTIONS_AVAILABLE])
//                 Context->Dr6 = 0x1F80;
//         }
//     }
//     else
//     {
//         /* For first thread in a Process */
//         Context->Eip = (ULONG)BaseProcessStartThunk;
//     }
//
// #elif defined(_M_AMD64)
//     DPRINT("BaseInitializeContext: %p\n", Context);
//     ASSERT(((ULONG_PTR)StackAddress & 15) == 0);
//
//     RtlZeroMemory(Context, sizeof(*Context));
//
//     /* Setup the Initial Win32 Thread Context */
//     Context->Rcx = (ULONG_PTR)StartAddress;
//     Context->Rdx = (ULONG_PTR)Parameter;
//     Context->Rsp = (ULONG_PTR)StackAddress - 5 * sizeof(PVOID);
//
//     /* Setup the Segments */
//     Context->SegGs = KGDT64_R3_DATA | RPL_MASK;
//     Context->SegEs = KGDT64_R3_DATA | RPL_MASK;
//     Context->SegDs = KGDT64_R3_DATA | RPL_MASK;
//     Context->SegCs = KGDT64_R3_CODE | RPL_MASK;
//     Context->SegSs = KGDT64_R3_DATA | RPL_MASK;
//     Context->SegFs = KGDT64_R3_CMTEB | RPL_MASK;
//
//     /* Set the EFLAGS */
//     Context->EFlags = 0x3000 | EFLAGS_INTERRUPT_MASK; /* IOPL 3 */
//
//     /* Set MXCSR */
//     Context->MxCsr = INITIAL_MXCSR;
//
//     if (ContextType == 1)      /* For Threads */
//     {
//         Context->Rip = (ULONG_PTR)BaseThreadStartup;
//     }
//     else if (ContextType == 2) /* For Fibers */
//     {
//         Context->Rip = (ULONG_PTR)BaseFiberStartup;
//     }
//     else                       /* For first thread in a Process */
//     {
//         Context->Rip = (ULONG_PTR)BaseProcessStartup;
//     }
//
//     /* Set the Context Flags */
//     Context->ContextFlags = CONTEXT_FULL;
// #elif defined(_M_ARM)
//     DPRINT("BaseInitializeContext: %p\n", Context);
//
//     // FIXME: check if this is correct!
//     /* Setup the Initial Win32 Thread Context */
//     Context->R0 = (ULONG_PTR)StartAddress;
//     Context->R1 = (ULONG_PTR)Parameter;
//     Context->Sp = (ULONG_PTR)StackAddress;
//
//     if (ContextType == 1)      /* For Threads */
//     {
//         Context->Pc = (ULONG_PTR)BaseThreadStartupThunk;
//     }
//     else if (ContextType == 2) /* For Fibers */
//     {
//         Context->Pc = (ULONG_PTR)BaseFiberStartup;
//     }
//     else                       /* For first thread in a Process */
//     {
//         Context->Pc = (ULONG_PTR)BaseProcessStartThunk;
//     }
//
//     /* Set the Context Flags */
//     Context->ContextFlags = CONTEXT_FULL;
//
//     /* Give it some room for the Parameter */
//     Context->Sp -= sizeof(PVOID);
// #elif defined(_M_ARM64)
//     DPRINT("BaseInitializeContext: %p\n", Context);
//
//     RtlZeroMemory(Context, sizeof(*Context));
//
//     /* Setup the Initial Win32 Thread Context (ARM64 calling convention) */
//     Context->X0 = (ULONG64)StartAddress;   /* 1st arg */
//     Context->X1 = (ULONG64)Parameter;      /* 2nd arg */
//     Context->Sp = (ULONG64)StackAddress;   /* Stack pointer (16-byte aligned) */
//     Context->Sp &= ~15ULL;                 /* Ensure alignment */
//
//     if (ContextType == 1)      /* For Threads */
//     {
//         Context->Pc = (ULONG64)BaseThreadStartup;
//     }
//     else if (ContextType == 2) /* For Fibers */
//     {
//         Context->Pc = (ULONG64)BaseFiberStartup;
//     }
//     else                       /* For first thread in a Process */
//     {
//         Context->Pc = (ULONG64)BaseProcessStartup;
//     }
//
//     /* Set the Context Flags */
//     Context->ContextFlags = CONTEXT_FULL;
// #else
// #warning Unknown architecture
//     UNIMPLEMENTED;
//     DbgBreakPoint();
// #endif
// }
//
// /*
//  * Checks if the privilege for Real-Time Priority is there
//  * Beware about this function behavior:
//  * - In case Keep is set to FALSE, then the function will only check
//  * whether real time is allowed and won't grant the privilege. In that case
//  * it will return TRUE if allowed, FALSE otherwise. Not a state!
//  * It means you don't have to release privilege when calling with FALSE.
//  */
// PVOID
// WINAPI
// BasepIsRealtimeAllowed(IN BOOLEAN Keep)
// {
//     ULONG Privilege = SE_INC_BASE_PRIORITY_PRIVILEGE;
//     PVOID State;
//     NTSTATUS Status;
//
//     Status = RtlAcquirePrivilege(&Privilege, 1, 0, &State);
//     if (!NT_SUCCESS(Status)) return NULL;
//
//     if (!Keep)
//     {
//         RtlReleasePrivilege(State);
//         State = (PVOID)TRUE;
//     }
//
//     return State;
// }
//
// /*
//  * Maps an image file into a section
//  */
// NTSTATUS
// WINAPI
// BasepMapFile(IN LPCWSTR lpApplicationName,
//              OUT PHANDLE hSection,
//              IN PUNICODE_STRING ApplicationName)
// {
//     RTL_RELATIVE_NAME_U RelativeName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE hFile = NULL;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     DPRINT("BasepMapFile\n");
//
//     /* Zero out the Relative Directory */
//     RelativeName.ContainingDirectory = NULL;
//
//     /* Find the application name */
//     if (!RtlDosPathNameToNtPathName_U(lpApplicationName,
//                                       ApplicationName,
//                                       NULL,
//                                       &RelativeName))
//     {
//         return STATUS_OBJECT_PATH_NOT_FOUND;
//     }
//
//     DPRINT("ApplicationName %wZ\n", ApplicationName);
//     DPRINT("RelativeName %wZ\n", &RelativeName.RelativeName);
//
//     /* Did we get a relative name? */
//     if (RelativeName.RelativeName.Length)
//     {
//         ApplicationName = &RelativeName.RelativeName;
//     }
//
//     /* Initialize the Object Attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                ApplicationName,
//                                OBJ_CASE_INSENSITIVE,
//                                RelativeName.ContainingDirectory,
//                                NULL);
//
//     /* Try to open the executable */
//     Status = NtOpenFile(&hFile,
//                         SYNCHRONIZE | FILE_EXECUTE | FILE_READ_DATA,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_DELETE | FILE_SHARE_READ,
//                         FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open file\n");
//         BaseSetLastNTError(Status);
//         return Status;
//     }
//
//     /* Create a section for this file */
//     Status = NtCreateSection(hSection,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              NULL,
//                              PAGE_EXECUTE,
//                              SEC_IMAGE,
//                              hFile);
//     NtClose(hFile);
//
//     /* Return status */
//     DPRINT("Section: %p for file: %p\n", *hSection, hFile);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// WINAPI
// Wow64EnableWow64FsRedirection(IN BOOLEAN Wow64EnableWow64FsRedirection)
// {
//     NTSTATUS Status;
//     BOOL Result;
//
//     Status = RtlWow64EnableFsRedirection(Wow64EnableWow64FsRedirection);
//     if (NT_SUCCESS(Status))
//     {
//         Result = TRUE;
//     }
//     else
//     {
//         BaseSetLastNTError(Status);
//         Result = FALSE;
//     }
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Wow64DisableWow64FsRedirection(IN PVOID *OldValue)
// {
//     NTSTATUS Status;
//     BOOL Result;
//
//     Status = RtlWow64EnableFsRedirectionEx((PVOID)TRUE, OldValue);
//     if (NT_SUCCESS(Status))
//     {
//         Result = TRUE;
//     }
//     else
//     {
//         BaseSetLastNTError(Status);
//         Result = FALSE;
//     }
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Wow64RevertWow64FsRedirection(IN PVOID OldValue)
// {
//     NTSTATUS Status;
//     BOOL Result;
//
//     Status = RtlWow64EnableFsRedirectionEx(OldValue, &OldValue);
//     if (NT_SUCCESS(Status))
//     {
//         Result = TRUE;
//     }
//     else
//     {
//         BaseSetLastNTError(Status);
//         Result = FALSE;
//     }
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// SetFileApisToOEM(VOID)
// {
//     /* Set the correct Base Api */
//     Basep8BitStringToUnicodeString = (PRTL_CONVERT_STRING)RtlOemStringToUnicodeString;
//     BasepUnicodeStringTo8BitString = RtlUnicodeStringToOemString;
//     BasepUnicodeStringTo8BitSize = BasepUnicodeStringToOemSize;
//     Basep8BitStringToUnicodeSize = BasepOemStringToUnicodeSize;
//
//     /* FIXME: Old, deprecated way */
//     bIsFileApiAnsi = FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// SetFileApisToANSI(VOID)
// {
//     /* Set the correct Base Api */
//     Basep8BitStringToUnicodeString = RtlAnsiStringToUnicodeString;
//     BasepUnicodeStringTo8BitString = RtlUnicodeStringToAnsiString;
//     BasepUnicodeStringTo8BitSize = BasepUnicodeStringToAnsiSize;
//     Basep8BitStringToUnicodeSize = BasepAnsiStringToUnicodeSize;
//
//     /* FIXME: Old, deprecated way */
//     bIsFileApiAnsi = TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// AreFileApisANSI(VOID)
// {
//    return Basep8BitStringToUnicodeString == RtlAnsiStringToUnicodeString;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// BaseMarkFileForDelete(IN HANDLE FileHandle,
//                       IN ULONG FileAttributes)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_BASIC_INFORMATION FileBasicInfo;
//     FILE_DISPOSITION_INFORMATION FileDispositionInfo;
//
//     /* If no attributes were given, get them */
//     if (!FileAttributes)
//     {
//         FileBasicInfo.FileAttributes = 0;
//         NtQueryInformationFile(FileHandle,
//                                &IoStatusBlock,
//                                &FileBasicInfo,
//                                sizeof(FileBasicInfo),
//                                FileBasicInformation);
//         FileAttributes = FileBasicInfo.FileAttributes;
//     }
//
//     /* If file is marked as RO, reset its attributes */
//     if (FileAttributes & FILE_ATTRIBUTE_READONLY)
//     {
//         RtlZeroMemory(&FileBasicInfo, sizeof(FileBasicInfo));
//         FileBasicInfo.FileAttributes = FILE_ATTRIBUTE_NORMAL;
//         NtSetInformationFile(FileHandle,
//                              &IoStatusBlock,
//                              &FileBasicInfo,
//                              sizeof(FileBasicInfo),
//                              FileBasicInformation);
//     }
//
//     /* Finally, mark the file for deletion */
//     FileDispositionInfo.DeleteFile = TRUE;
//     NtSetInformationFile(FileHandle,
//                          &IoStatusBlock,
//                          &FileDispositionInfo,
//                          sizeof(FileDispositionInfo),
//                          FileDispositionInformation);
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// WINAPI
// BasepCheckWinSaferRestrictions(IN HANDLE UserToken,
//                                IN LPWSTR ApplicationName,
//                                IN HANDLE FileHandle,
//                                OUT PBOOLEAN InJob,
//                                OUT PHANDLE NewToken,
//                                OUT PHANDLE JobHandle)
// {
//     NTSTATUS Status;
//
//     /* Validate that there's a name */
//     if ((ApplicationName) && *(ApplicationName))
//     {
//         /* Validate that the required output parameters are there */
//         if ((InJob) && (NewToken) && (JobHandle))
//         {
//             /* Do the work (one day...) */
//             DPRINT("BasepCheckWinSaferRestrictions is UNIMPLEMENTED\n");
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* Act as if SEH hit this */
//             Status = STATUS_ACCESS_VIOLATION;
//         }
//     }
//     else
//     {
//         /* Input is invalid */
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
