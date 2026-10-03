use core::ffi::c_void;
use ntdll_sys::nt::*;

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


// CRT helpers — cdecl exports from ntdll. extern "C", NOT "system": on i386
// extern "system" = stdcall, which would mis-decorate these cdecl symbols.
extern "C" {
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    fn wcsncpy(dest: *mut u16, src: *const u16, count: usize) -> *mut u16;
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    fn _wcsupr(s: *mut u16) -> *mut u16;
    fn _stricmp(s1: PCHAR, s2: PCHAR) -> core::ffi::c_int;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> core::ffi::c_int;
    fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> core::ffi::c_int;
}

// RtlGetProcessHeap is a PEB inline (TEB->Peb->ProcessHeap), not an export.
#[inline]
unsafe fn RtlGetProcessHeap() -> *mut c_void {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

#[inline]
unsafe fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

const STATUS_INVALID_IMPORT_OF_NON_DLL: NTSTATUS = 0xC000036Fu32 as NTSTATUS;

// RtlCopyMemory is a memmove macro, not an export.
#[inline]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// RtlInitEmptyUnicodeString / RtlInitEmptyAnsiString are FORCEINLINE in the
// headers (bindgen emits them as externs with no symbol). Shadow the glob.
#[inline]
unsafe fn RtlInitEmptyUnicodeString(s: PUNICODE_STRING, buffer: PWCHAR, buffer_size: USHORT) {
    (*s).Length = 0;
    (*s).MaximumLength = buffer_size;
    (*s).Buffer = buffer;
}

#[inline]
unsafe fn RtlInitEmptyAnsiString(s: PANSI_STRING, buffer: PCHAR, buffer_size: USHORT) {
    (*s).Length = 0;
    (*s).MaximumLength = buffer_size;
    (*s).Buffer = buffer;
}

// ntstrsafe.h functions are `static __inline NTSTATUS NTAPI` — not exports.
#[inline]
unsafe fn RtlStringCbLengthW(psz: *const u16, cb_max: usize, pcb: *mut usize) -> NTSTATUS {
    let max_chars = cb_max / core::mem::size_of::<u16>();
    let mut n = 0usize;
    while n < max_chars && *psz.add(n) != 0 {
        n += 1;
    }
    if n >= max_chars {
        return 0xC0000023u32 as NTSTATUS;
    }
    if !pcb.is_null() {
        *pcb = n * core::mem::size_of::<u16>();
    }
    0
}

#[inline]
unsafe fn RtlStringCbCopyNW(
    dest: *mut u16,
    cb_dest: usize,
    src: *const u16,
    cb_to_copy: usize,
) -> NTSTATUS {
    let dest_chars = cb_dest / core::mem::size_of::<u16>();
    if dest_chars == 0 {
        return 0xC0000023u32 as NTSTATUS;
    }
    let copy_chars = (cb_to_copy / core::mem::size_of::<u16>()).min(dest_chars - 1);
    let mut i = 0usize;
    while i < copy_chars && *src.add(i) != 0 {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    *dest.add(i) = 0;
    0
}

// InitializeObjectAttributes macro → inline.
#[inline]
unsafe fn InitializeObjectAttributes_inline(
    p: POBJECT_ATTRIBUTES,
    n: PUNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

// ReactOS wide-string literal helper → stable 'static NUL-terminated *mut u16.
// Inputs are ASCII; converted at const-eval time.
macro_rules! wstr {
    ($s:literal) => {{
        const BYTES: &[u8] = $s.as_bytes();
        const N: usize = BYTES.len() + 1;
        static ARR: [u16; N] = {
            let mut arr = [0u16; N];
            let mut i = 0;
            while i < BYTES.len() {
                arr[i] = BYTES[i] as u16;
                i += 1;
            }
            arr
        };
        ARR.as_ptr() as PWSTR
    }};
}

// RTL_QUERY_REGISTRY_ROUTINE. This used to bridge a nominal-type gap by TRANSMUTE: bindgen
// typed the table field with the arch's literal convention (stdcall on i386, C on 64-bit)
// while the translated routines are `system`, so a per-arch `CfgRoutineApi` alias existed
// purely to be transmuted into. Same ABI, but the transmute erased the whole fn type — a
// genuine signature mismatch here would have been silently accepted.
//
// normalize_island_abi.py spells that slot `system`, so the assignment type-checks directly
// and the alias and the transmute are both gone.
type CfgRoutineSys =
    unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS;

const fn qr(f: CfgRoutineSys) -> PRTL_QUERY_REGISTRY_ROUTINE {
    Some(f)
}

// ---- file-scope preamble (translated) ----
pub static mut SmpSubsystemName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
pub static mut PosixName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
pub static mut Os2Name: _UNICODE_STRING = unsafe { core::mem::zeroed() };

pub static mut SmpBootExecuteList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SmpSetupExecuteList: LIST_ENTRY = unsafe { core::mem::zeroed() };

pub static mut SmpPagingFileList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut SmpDosDevicesList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut SmpFileRenameList: LIST_ENTRY = unsafe { core::mem::zeroed() };

pub static mut SmpKnownDllsList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut SmpExcludeKnownDllsList: LIST_ENTRY = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut SmpSubSystemList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SmpSubSystemsToLoad: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SmpSubSystemsToDefer: LIST_ENTRY = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut SmpExecuteList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut NativeProcessList: LIST_ENTRY = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut SmpHeap: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut SmBaseTag: ULONG = 0;
pub static mut SmpDebugPort: HANDLE = core::ptr::null_mut();
pub static mut SmpDosDevicesObjectDirectory: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut SmpDefaultEnvironment: PWCHAR = core::ptr::null_mut();
pub static mut SmpDefaultLibPathBuffer: PWCHAR = core::ptr::null_mut();

pub static mut SmpKnownDllPath: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SmpDefaultLibPath: _UNICODE_STRING = unsafe { core::mem::zeroed() };

pub static mut SmpCalledConfigEnv: ULONG = 0;
pub static mut SmpInitProgressByLine: ULONG = 0;
pub static mut SmpInitReturnStatus: NTSTATUS = 0;
pub static mut SmpInitLastCall: PVOID = core::ptr::null_mut();

pub static mut SmpPrimarySDBody: _SECURITY_DESCRIPTOR = unsafe { core::mem::zeroed() };
pub static mut SmpLiberalSDBody: _SECURITY_DESCRIPTOR = unsafe { core::mem::zeroed() };
pub static mut SmpKnownDllsSDBody: _SECURITY_DESCRIPTOR = unsafe { core::mem::zeroed() };
pub static mut SmpApiPortSDBody: _SECURITY_DESCRIPTOR = unsafe { core::mem::zeroed() };

pub static mut SmpPrimarySecurityDescriptor: PISECURITY_DESCRIPTOR = core::ptr::null_mut();
pub static mut SmpLiberalSecurityDescriptor: PISECURITY_DESCRIPTOR = core::ptr::null_mut();
pub static mut SmpKnownDllsSecurityDescriptor: PISECURITY_DESCRIPTOR = core::ptr::null_mut();
pub static mut SmpApiPortSecurityDescriptor: PISECURITY_DESCRIPTOR = core::ptr::null_mut();

pub static mut SmpAllowProtectedRenames: ULONG = 0;
pub static mut SmpProtectionMode: ULONG = 1;
pub static mut MiniNTBoot: BOOLEAN = 0u8;

pub static mut SmpRegistryConfigurationTable: [RTL_QUERY_REGISTRY_TABLE; 21] = [
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureProtectionMode),
        Flags: 0,
        Name: wstr!("ProtectionMode\0"),
        EntryContext: core::ptr::null_mut(),
        DefaultType: REG_DWORD,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureAllowProtectedRenames),
        Flags: RTL_QUERY_REGISTRY_DELETE,
        Name: wstr!("AllowProtectedRenames\0"),
        EntryContext: core::ptr::null_mut(),
        DefaultType: REG_DWORD,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureObjectDirectories),
        Flags: 0,
        Name: wstr!("ObjectDirectories\0"),
        EntryContext: core::ptr::null_mut(),
        DefaultType: REG_MULTI_SZ,
        DefaultData: wstr!("\\Windows\0\\RPC Control\0") as *mut core::ffi::c_void,
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureExecute),
        Flags: 0,
        Name: wstr!("BootExecute\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpBootExecuteList) as *mut core::ffi::c_void,
        DefaultType: REG_MULTI_SZ,
        DefaultData: wstr!("autocheck AutoChk.exe *\0") as *mut core::ffi::c_void,
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureExecute),
        Flags: RTL_QUERY_REGISTRY_TOPKEY,
        Name: wstr!("SetupExecute\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpSetupExecuteList) as *mut core::ffi::c_void,
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureFileRenames),
        Flags: RTL_QUERY_REGISTRY_DELETE,
        Name: wstr!("PendingFileRenameOperations\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpFileRenameList) as *mut core::ffi::c_void,
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureFileRenames),
        Flags: RTL_QUERY_REGISTRY_DELETE,
        Name: wstr!("PendingFileRenameOperations2\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpFileRenameList) as *mut core::ffi::c_void,
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureExcludeKnownDlls),
        Flags: 0,
        Name: wstr!("ExcludeFromKnownDlls\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpExcludeKnownDllsList) as *mut core::ffi::c_void,
        DefaultType: REG_MULTI_SZ,
        DefaultData: wstr!("\0") as *mut core::ffi::c_void,
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: None,
        Flags: RTL_QUERY_REGISTRY_SUBKEY,
        Name: wstr!("Memory Management\0"),
        EntryContext: core::ptr::null_mut(),
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureMemoryMgmt),
        Flags: 0,
        Name: wstr!("PagingFiles\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpPagingFileList) as *mut core::ffi::c_void,
        DefaultType: REG_MULTI_SZ,
        DefaultData: wstr!("?:\\pagefile.sys\0") as *mut core::ffi::c_void,
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureDosDevices),
        Flags: RTL_QUERY_REGISTRY_SUBKEY,
        Name: wstr!("DOS Devices\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpDosDevicesList) as *mut core::ffi::c_void,
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureKnownDlls),
        Flags: RTL_QUERY_REGISTRY_SUBKEY,
        Name: wstr!("KnownDlls\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpKnownDllsList) as *mut core::ffi::c_void,
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureEnvironment),
        Flags: RTL_QUERY_REGISTRY_SUBKEY,
        Name: wstr!("Environment\0"),
        EntryContext: core::ptr::null_mut(),
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureEnvironment),
        Flags: RTL_QUERY_REGISTRY_SUBKEY,
        Name: wstr!("Environment\0"),
        EntryContext: core::ptr::null_mut(),
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureSubSystems),
        Flags: RTL_QUERY_REGISTRY_SUBKEY,
        Name: wstr!("SubSystems\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpSubSystemList) as *mut core::ffi::c_void,
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureSubSystems),
        Flags: RTL_QUERY_REGISTRY_NOEXPAND,
        Name: wstr!("Required\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpSubSystemList) as *mut core::ffi::c_void,
        DefaultType: REG_MULTI_SZ,
        DefaultData: wstr!("Debug\0Windows\0") as *mut core::ffi::c_void,
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureSubSystems),
        Flags: RTL_QUERY_REGISTRY_NOEXPAND,
        Name: wstr!("Optional\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpSubSystemList) as *mut core::ffi::c_void,
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureSubSystems),
        Flags: 0,
        Name: wstr!("Kmode\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpSubSystemList) as *mut core::ffi::c_void,
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: qr(SmpConfigureExecute),
        Flags: RTL_QUERY_REGISTRY_TOPKEY,
        Name: wstr!("Execute\0"),
        EntryContext: core::ptr::addr_of_mut!(SmpExecuteList) as *mut core::ffi::c_void,
        DefaultType: REG_NONE,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: None,
        Flags: 0,
        Name: core::ptr::null_mut(),
        EntryContext: core::ptr::null_mut(),
        DefaultType: 0,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    },
    // Sentinel terminator entry (rustconv dropped the trailing {0}).
    unsafe { core::mem::zeroed() },
];

#[no_mangle]
pub unsafe extern "system" fn SmpSaveRegistryValue(
    ListAddress: PLIST_ENTRY,
    Name: PWSTR,
    Value: *mut u16,
    Flags: u8,
) -> i32 {
    let mut RegEntry: PSMP_REGISTRY_VALUE = core::ptr::null_mut();
    let mut NameString: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueString: _UNICODE_STRING = core::mem::zeroed();
    let mut AnsiValueString: ANSI_STRING = core::mem::zeroed();
    let mut NextEntry: PLIST_ENTRY;

    RtlInitUnicodeString(&mut NameString, Name);
    RtlInitUnicodeString(&mut ValueString, Value);

    if Flags != 0 {
        NextEntry = (*ListAddress).Flink;
        while NextEntry != ListAddress {
            RegEntry = (NextEntry as *mut u8)
                .offset(-(core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry) as isize))
                as PSMP_REGISTRY_VALUE;

            if RtlCompareUnicodeString(&mut (*RegEntry).Name, &mut NameString, 1u8) == 0 {
                if RtlCompareUnicodeString(&mut (*RegEntry).Value, &mut ValueString, 1u8) == 0 {
                    return 0x40000000u32 as i32;
                }
                break;
            }

            NextEntry = (*NextEntry).Flink;
            RegEntry = core::ptr::null_mut();
        }
    }

    if RegEntry.is_null() {
        RegEntry = RtlAllocateHeap(
            RtlGetProcessHeap(),
            SmBaseTag,
            (core::mem::size_of::<SMP_REGISTRY_VALUE>() + NameString.MaximumLength as usize) as _,
        ) as PSMP_REGISTRY_VALUE;
        if RegEntry.is_null() {
            return STATUS_NO_MEMORY;
        }

        InitializeListHead(core::ptr::addr_of_mut!((*RegEntry).Entry));
        (*RegEntry).AnsiValue = core::ptr::null_mut();
        (*RegEntry).Value.Buffer = core::ptr::null_mut();

        (*RegEntry).Name.Buffer = RegEntry.offset(1) as *mut u16;
        (*RegEntry).Name.Length = NameString.Length;
        (*RegEntry).Name.MaximumLength = NameString.MaximumLength;
        RtlCopyMemory(
            (*RegEntry).Name.Buffer as *mut core::ffi::c_void,
            NameString.Buffer as *const core::ffi::c_void,
            NameString.MaximumLength as usize,
        );

        InsertTailList(ListAddress, core::ptr::addr_of_mut!((*RegEntry).Entry));
    }

    if !(*RegEntry).Value.Buffer.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            (*RegEntry).Value.Buffer as *mut core::ffi::c_void,
        );
    }

    if Value.is_null() {
        RtlInitUnicodeString(&mut (*RegEntry).Value, core::ptr::null_mut());
        return STATUS_SUCCESS;
    }

    (*RegEntry).Value.Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        SmBaseTag,
        ValueString.MaximumLength as _,
    ) as *mut u16;
    if (*RegEntry).Value.Buffer.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*RegEntry).Entry));
        RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry as *mut core::ffi::c_void);
        return STATUS_NO_MEMORY;
    }

    (*RegEntry).Value.Length = ValueString.Length;
    (*RegEntry).Value.MaximumLength = ValueString.MaximumLength;
    RtlCopyMemory(
        (*RegEntry).Value.Buffer as *mut core::ffi::c_void,
        ValueString.Buffer as *const core::ffi::c_void,
        ValueString.MaximumLength as usize,
    );

    (*RegEntry).AnsiValue = RtlAllocateHeap(
        RtlGetProcessHeap(),
        SmBaseTag,
        ((ValueString.Length as usize / core::mem::size_of::<u16>()) + 1) as _,
    ) as *mut i8;
    if (*RegEntry).AnsiValue.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            (*RegEntry).Value.Buffer as *mut core::ffi::c_void,
        );
        RemoveEntryList(core::ptr::addr_of_mut!((*RegEntry).Entry));
        RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry as *mut core::ffi::c_void);
        return STATUS_NO_MEMORY;
    }

    RtlInitEmptyAnsiString(
        &mut AnsiValueString,
        (*RegEntry).AnsiValue,
        ((ValueString.Length as usize / core::mem::size_of::<u16>()) + 1) as u16,
    );
    RtlUnicodeStringToAnsiString(&mut AnsiValueString, &mut ValueString, 0u8);
    STATUS_SUCCESS
}

unsafe extern "system" fn SmpFindRegistryValue(
    List: PLIST_ENTRY,
    ValueName: PWSTR,
) -> PSMP_REGISTRY_VALUE {
    let mut RegEntry: PSMP_REGISTRY_VALUE = core::ptr::null_mut();
    let mut ValueString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut ValueString, ValueName);

    let mut NextEntry = (*List).Flink;
    while NextEntry != List {
        RegEntry = (NextEntry as *mut u8)
            .offset(-(core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry) as isize))
            as PSMP_REGISTRY_VALUE;

        if RtlCompareUnicodeString(&mut (*RegEntry).Name, &mut ValueString, 1u8) == 0 {
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry == List {
        RegEntry = core::ptr::null_mut();
    }
    RegEntry
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureProtectionMode(
    _ValueName: PWSTR,
    _ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    _Context: *mut core::ffi::c_void,
    _EntryContext: *mut core::ffi::c_void,
) -> NTSTATUS {
    if ValueLength as usize == core::mem::size_of::<u32>() {
        SmpProtectionMode = *(ValueData as *mut u32);
    } else {
        SmpProtectionMode = 1;
    }

    SmpCreateSecurityDescriptors(0u8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureAllowProtectedRenames(
    _ValueName: PWSTR,
    _ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    _Context: PVOID,
    _EntryContext: PVOID,
) -> NTSTATUS {
    if ValueLength as usize == core::mem::size_of::<ULONG>() {
        SmpAllowProtectedRenames = *(ValueData as *mut ULONG);
    } else {
        SmpAllowProtectedRenames = 0;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureObjectDirectories(
    _ValueName: PWSTR,
    _ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    _ValueLength: u32,
    _Context: *mut core::ffi::c_void,
    _EntryContext: *mut core::ffi::c_void,
) -> i32 {
    let mut SecDescriptor: PISECURITY_DESCRIPTOR;
    let mut Status: i32;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DirHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut RpcString: _UNICODE_STRING = core::mem::zeroed();
    let mut WindowsString: _UNICODE_STRING = core::mem::zeroed();
    let mut SearchString: _UNICODE_STRING = core::mem::zeroed();
    let mut SourceString: PWSTR = ValueData as PWSTR;

    RtlInitUnicodeString(&mut RpcString, wstr!("\\RPC Control\0"));
    RtlInitUnicodeString(&mut WindowsString, wstr!("\\Windows\0"));

    while *SourceString != 0 {
        RtlInitUnicodeString(&mut SearchString, SourceString);
        SecDescriptor = SmpPrimarySecurityDescriptor;

        if RtlEqualUnicodeString(&mut SearchString, &mut RpcString, 1u8) != 0
            || RtlEqualUnicodeString(&mut SearchString, &mut WindowsString, 1u8) != 0
        {
            SecDescriptor = SmpLiberalSecurityDescriptor;
        }

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT) as u32;
        ObjectAttributes.ObjectName = &mut SearchString;
        ObjectAttributes.SecurityDescriptor = SecDescriptor as *mut core::ffi::c_void;
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status =
            NtCreateDirectoryObject(&mut DirHandle, DIRECTORY_ALL_ACCESS, &mut ObjectAttributes);
        if !NT_SUCCESS(Status) {
        } else {
            NtClose(DirHandle);
        }

        SourceString = SourceString.add(wcslen(SourceString) + 1);
    }

    STATUS_SUCCESS
}

/// The *Execute values' handler (upstream 41dadd942f7): like SmpConfigureMemoryMgmt, but an
/// invalid or EMPTY string is skipped instead of being queued and later run as a command.
#[no_mangle]
pub unsafe extern "system" fn SmpConfigureExecute(
    _ValueName: PWSTR,
    _ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    _Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    let mut StrLength: usize = 0;
    let Status = RtlStringCbLengthW(ValueData as *const u16, ValueLength as usize, &mut StrLength);
    if !NT_SUCCESS(Status) || StrLength < core::mem::size_of::<u16>() {
        return STATUS_SUCCESS;
    }
    SmpSaveRegistryValue(
        EntryContext as PLIST_ENTRY,
        ValueData as PWSTR,
        core::ptr::null_mut(),
        1u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureMemoryMgmt(
    _ValueName: PWSTR,
    _ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    _ValueLength: u32,
    _Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    SmpSaveRegistryValue(
        EntryContext as PLIST_ENTRY,
        ValueData as PWSTR,
        core::ptr::null_mut(),
        1u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureFileRenames(
    _ValueName: PWSTR,
    _ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    _ValueLength: u32,
    _Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    static mut Canary: *mut u16 = core::ptr::null_mut();
    let Status: i32;

    if !Canary.is_null() {
        Status = SmpSaveRegistryValue(
            EntryContext as PLIST_ENTRY,
            Canary,
            ValueData as *mut u16,
            0u8,
        );
        Canary = core::ptr::null_mut();
    } else {
        Canary = ValueData as *mut u16;
        Status = STATUS_SUCCESS;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureExcludeKnownDlls(
    _ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    _ValueLength: ULONG,
    _Context: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    let mut DllName: PWCHAR;
    let mut Status: NTSTATUS;

    if (ValueType == REG_MULTI_SZ) || (ValueType == REG_SZ) {
        DllName = ValueData as PWCHAR;
        while *DllName != 0 {
            Status = SmpSaveRegistryValue(
                EntryContext as PLIST_ENTRY,
                DllName,
                core::ptr::null_mut(),
                1u8,
            );

            if !NT_SUCCESS(Status) || (ValueType == REG_SZ) {
                return Status;
            }

            DllName = DllName.add(wcslen(DllName) + 1);
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureDosDevices(
    ValueName: PWSTR,
    _ValueType: ULONG,
    ValueData: PVOID,
    _ValueLength: ULONG,
    _Context: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    SmpSaveRegistryValue(
        EntryContext as PLIST_ENTRY,
        ValueName,
        ValueData as *mut u16,
        1u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SmpInitializeKnownDllPath(
    DllPath: PUNICODE_STRING,
    Buffer: *mut u16,
    Length: u32,
) -> i32 {
    let Status: i32;

    (*DllPath).Buffer = RtlAllocateHeap(RtlGetProcessHeap(), SmBaseTag, Length as _) as *mut u16;
    if !(*DllPath).Buffer.is_null() {
        (*DllPath).MaximumLength = Length as u16;
        (*DllPath).Length = (Length as u16).wrapping_sub(core::mem::size_of::<u16>() as u16);

        RtlCopyMemory(
            (*DllPath).Buffer as *mut core::ffi::c_void,
            Buffer as *const core::ffi::c_void,
            Length as usize,
        );
        Status = STATUS_SUCCESS;
    } else {
        Status = STATUS_NO_MEMORY;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureKnownDlls(
    ValueName: PWSTR,
    _ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    _Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    if _wcsicmp(
        ValueName,
        w!("DllDirectory").as_ptr() as *const u16,
    ) == 0
    {
        SmpInitializeKnownDllPath(
            core::ptr::addr_of_mut!(SmpKnownDllPath),
            ValueData as *mut u16,
            ValueLength,
        )
    } else {
        SmpSaveRegistryValue(
            EntryContext as PLIST_ENTRY,
            ValueName,
            ValueData as *mut u16,
            1u8,
        )
    }
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureEnvironment(
    ValueName: PWSTR,
    _ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    _Context: PVOID,
    _EntryContext: PVOID,
) -> NTSTATUS {
    let mut ValueString: _UNICODE_STRING = core::mem::zeroed();
    let mut DataString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut ValueString, ValueName);
    RtlInitUnicodeString(&mut DataString, ValueData as PWSTR);
    let Status =
        RtlSetEnvironmentVariable(core::ptr::null_mut(), &mut ValueString, &mut DataString);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if _wcsicmp(ValueName, w!("Path").as_ptr() as PWSTR) == 0 {
        SmpCalledConfigEnv += 1;
        if SmpCalledConfigEnv == 2 {
            SmpDefaultLibPathBuffer =
                RtlAllocateHeap(RtlGetProcessHeap(), SmBaseTag, ValueLength as _) as PWCHAR;
            if SmpDefaultLibPathBuffer.is_null() {
                return STATUS_NO_MEMORY;
            }

            RtlCopyMemory(
                SmpDefaultLibPathBuffer as PVOID,
                ValueData,
                ValueLength as usize,
            );
            RtlInitUnicodeString(&raw mut SmpDefaultLibPath, SmpDefaultLibPathBuffer);
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SmpConfigureSubSystems(
    ValueName: PWSTR,
    ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    _ValueLength: u32,
    _Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    let mut RegEntry: PSMP_REGISTRY_VALUE;
    let mut SubsystemName: PWCHAR;

    if (_wcsicmp(
        ValueName,
        w!("Required").as_ptr() as *const u16,
    ) != 0)
        && (_wcsicmp(
            ValueName,
            w!("Optional").as_ptr() as *const u16,
        ) != 0)
    {
        if (_wcsicmp(
            ValueName,
            w!("PosixSingleInstance").as_ptr() as *const u16,
        ) != 0)
            || (ValueType != REG_DWORD)
        {
            return SmpSaveRegistryValue(
                EntryContext as PLIST_ENTRY,
                ValueName,
                ValueData as *mut u16,
                1u8,
            );
        }

        RegPosixSingleInstance = 1u8;
        return STATUS_SUCCESS as i32;
    }

    if ValueType == REG_MULTI_SZ {
        SubsystemName = ValueData as PWCHAR;
        while *SubsystemName != 0 {
            RegEntry = SmpFindRegistryValue(EntryContext as PLIST_ENTRY, SubsystemName);
            if RegEntry.is_null() {
                // Invalid subsystem name, skip
            } else {
                RemoveEntryList(core::ptr::addr_of_mut!((*RegEntry).Entry));

                if _wcsicmp(
                    ValueName,
                    w!("Required").as_ptr() as *const u16,
                ) == 0
                {
                    InsertTailList(
                        core::ptr::addr_of_mut!(SmpSubSystemsToLoad),
                        core::ptr::addr_of_mut!((*RegEntry).Entry),
                    );
                } else {
                    InsertTailList(
                        core::ptr::addr_of_mut!(SmpSubSystemsToDefer),
                        core::ptr::addr_of_mut!((*RegEntry).Entry),
                    );
                }
            }

            SubsystemName = SubsystemName.add(wcslen(SubsystemName) + 1);
        }
    }

    STATUS_SUCCESS as i32
}

#[no_mangle]
pub unsafe extern "system" fn SmpTranslateSystemPartitionInformation() {
    let mut Status;
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut LinkTarget: _UNICODE_STRING = core::mem::zeroed();
    let mut SearchString: _UNICODE_STRING = core::mem::zeroed();
    let mut SystemPartition: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle = core::ptr::null_mut();
    let mut LinkHandle = core::ptr::null_mut();
    let mut Length: u32 = 0;
    let mut Context: u32 = 0;
    let mut StrLength: usize = 0;
    let mut LinkBuffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    // Header-typed buffers, so the structs read out of them are aligned (upstream
    // aa8d624253a); a bare i8 array is 1-aligned, which makes every field access UB.
    #[repr(C)]
    struct ValueBufferT {
        Info: KEY_VALUE_PARTIAL_INFORMATION,
        Buffer: [i8; 512],
    }
    #[repr(C)]
    struct DirInfoBufferT {
        Info: OBJECT_DIRECTORY_INFORMATION,
        Buffer: [u16; 256],
    }
    let mut ValueBuffer: ValueBufferT = core::mem::zeroed();
    let PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::addr_of_mut!(ValueBuffer.Info);
    let mut DirInfoBuffer: DirInfoBufferT = core::mem::zeroed();
    let DirInfo: POBJECT_DIRECTORY_INFORMATION = core::ptr::addr_of_mut!(DirInfoBuffer.Info);

    /* Open the setup key */
    RtlInitUnicodeString(
        &mut UnicodeString,
        w!("\\Registry\\Machine\\System\\Setup").as_ptr() as *const u16,
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut UnicodeString;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenKey(&mut KeyHandle, KEY_READ, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return;
    }

    /* Query the system partition */
    RtlInitUnicodeString(
        &mut UnicodeString,
        w!("SystemPartition").as_ptr() as *const u16,
    );
    Status = NtQueryValueKey(
        KeyHandle,
        &mut UnicodeString,
        KeyValuePartialInformation,
        PartialInfo as *mut _,
        core::mem::size_of_val(&ValueBuffer) as u32,
        &mut Length,
    );
    NtClose(KeyHandle);
    if !NT_SUCCESS(Status)
        || ((*PartialInfo).Type != REG_SZ && (*PartialInfo).Type != REG_EXPAND_SZ)
    {
        return;
    }

    /* Initialize the system partition string */
    RtlInitEmptyUnicodeString(
        &mut SystemPartition,
        core::ptr::addr_of_mut!((*PartialInfo).Data) as *mut u16,
        (*PartialInfo).DataLength as u16,
    );
    RtlStringCbLengthW(
        SystemPartition.Buffer,
        SystemPartition.MaximumLength as usize,
        &mut StrLength,
    );
    SystemPartition.Length = StrLength as u16;

    /* Enumerate the directory looking for the symbolic link string */
    RtlInitUnicodeString(
        &mut SearchString,
        w!("SymbolicLink").as_ptr() as *const u16,
    );
    RtlInitEmptyUnicodeString(
        &mut LinkTarget,
        LinkBuffer.as_mut_ptr(),
        core::mem::size_of_val(&LinkBuffer) as u16,
    );
    Status = NtQueryDirectoryObject(
        SmpDosDevicesObjectDirectory,
        DirInfo as *mut _,
        core::mem::size_of_val(&DirInfoBuffer) as u32,
        1u8,
        1u8,
        &mut Context,
        core::ptr::null_mut(),
    );

    /* Keep searching until we find it */
    while NT_SUCCESS(Status) {
        if RtlEqualUnicodeString(&mut (*DirInfo).TypeName, &mut SearchString, 1u8) != 0
            && (*DirInfo).Name.Length as usize == 2 * core::mem::size_of::<u16>()
            && *(*DirInfo).Name.Buffer.add(1) == b':' as u16
        {
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            ObjectAttributes.RootDirectory = SmpDosDevicesObjectDirectory;
            ObjectAttributes.ObjectName = &mut (*DirInfo).Name;
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
            Status = NtOpenSymbolicLinkObject(
                &mut LinkHandle,
                SYMBOLIC_LINK_ALL_ACCESS,
                &mut ObjectAttributes,
            );
            if NT_SUCCESS(Status) {
                Status =
                    NtQuerySymbolicLinkObject(LinkHandle, &mut LinkTarget, core::ptr::null_mut());
                NtClose(LinkHandle);

                if NT_SUCCESS(Status)
                    && (RtlEqualUnicodeString(&mut SystemPartition, &mut LinkTarget, 1u8) != 0
                        || (RtlPrefixUnicodeString(&mut SystemPartition, &mut LinkTarget, 1u8)
                            != 0
                            && *LinkTarget.Buffer.add(
                                SystemPartition.Length as usize / core::mem::size_of::<u16>(),
                            ) == b'\\' as u16))
                {
                    break;
                }
            }
        }

        Status = NtQueryDirectoryObject(
            SmpDosDevicesObjectDirectory,
            DirInfo as *mut _,
            core::mem::size_of_val(&DirInfoBuffer) as u32,
            1u8,
            0u8,
            &mut Context,
            core::ptr::null_mut(),
        );
    }
    if !NT_SUCCESS(Status) {
        // No drive letter maps the system partition (the normal case for a UEFI ESP): fall
        // back to the OS boot drive's letter rather than writing no BootDir at all (upstream
        // c498d0930a7, with aa8d624253a pointing Name.Buffer at storage first -- the original
        // fallback wrote through whatever the last failed query left there). Any other
        // failure still gives up.
        if Status != STATUS_NO_MORE_ENTRIES {
            return;
        }
        (*DirInfo).Name.Buffer = DirInfoBuffer.Buffer.as_mut_ptr();
        *(*DirInfo).Name.Buffer = (*crate::smss::smutil::SharedUserData).NtSystemRoot[0];
        *(*DirInfo).Name.Buffer.add(1) = (*crate::smss::smutil::SharedUserData).NtSystemRoot[1];
    }

    /* Open the setup key again, for full access this time */
    RtlInitUnicodeString(
        &mut UnicodeString,
        w!("\\Registry\\Machine\\Software\\Microsoft\\Windows\\CurrentVersion\\Setup").as_ptr() as *const u16,
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut UnicodeString;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenKey(&mut KeyHandle, KEY_ALL_ACCESS, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return;
    }

    /* Wrap up the end of the link buffer */
    wcsncpy(LinkBuffer.as_mut_ptr(), (*DirInfo).Name.Buffer, 2);
    LinkBuffer[2] = b'\\' as u16;
    LinkBuffer[3] = 0;

    /* Now set this as the "BootDir" */
    RtlInitUnicodeString(
        &mut UnicodeString,
        w!("BootDir").as_ptr() as *const u16,
    );
    Status = NtSetValueKey(
        KeyHandle,
        &mut UnicodeString,
        0,
        REG_SZ,
        LinkBuffer.as_mut_ptr() as *mut _,
        (4 * core::mem::size_of::<u16>()) as u32,
    );
    let _ = Status;
    NtClose(KeyHandle);
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreateSecurityDescriptors(InitialCall: u8) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut WorldSid: PSID = core::ptr::null_mut();
    let mut AdminSid: PSID = core::ptr::null_mut();
    let mut SystemSid: PSID = core::ptr::null_mut();
    let mut RestrictedSid: PSID = core::ptr::null_mut();
    let mut OwnerSid: PSID = core::ptr::null_mut();
    let mut WorldAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 1],
    };
    let mut NtAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let mut CreatorAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 3],
    };
    let mut AclLength: ULONG;
    let mut SidLength: ULONG;
    let mut Acl: PACL;
    let mut Ace: PACE_HEADER = core::ptr::null_mut();
    let mut ProtectionRequired: u8 = 0u8;

    'quickie: loop {
        if InitialCall != 0 {
            SmpPrimarySecurityDescriptor = &raw mut SmpPrimarySDBody as *mut _ as PISECURITY_DESCRIPTOR;
            RtlCreateSecurityDescriptor(
                SmpPrimarySecurityDescriptor as *mut _,
                SECURITY_DESCRIPTOR_REVISION as ULONG,
            );
            RtlSetDaclSecurityDescriptor(
                SmpPrimarySecurityDescriptor as *mut _,
                1u8,
                core::ptr::null_mut(),
                0u8,
            );

            SmpLiberalSecurityDescriptor = &raw mut SmpLiberalSDBody as *mut _ as PISECURITY_DESCRIPTOR;
            RtlCreateSecurityDescriptor(
                SmpLiberalSecurityDescriptor as *mut _,
                SECURITY_DESCRIPTOR_REVISION as ULONG,
            );
            RtlSetDaclSecurityDescriptor(
                SmpLiberalSecurityDescriptor as *mut _,
                1u8,
                core::ptr::null_mut(),
                0u8,
            );

            SmpKnownDllsSecurityDescriptor =
                &raw mut SmpKnownDllsSDBody as *mut _ as PISECURITY_DESCRIPTOR;
            RtlCreateSecurityDescriptor(
                SmpKnownDllsSecurityDescriptor as *mut _,
                SECURITY_DESCRIPTOR_REVISION as ULONG,
            );
            RtlSetDaclSecurityDescriptor(
                SmpKnownDllsSecurityDescriptor as *mut _,
                1u8,
                core::ptr::null_mut(),
                0u8,
            );

            SmpApiPortSecurityDescriptor = &raw mut SmpApiPortSDBody as *mut _ as PISECURITY_DESCRIPTOR;
            RtlCreateSecurityDescriptor(
                SmpApiPortSecurityDescriptor as *mut _,
                SECURITY_DESCRIPTOR_REVISION as ULONG,
            );
            RtlSetDaclSecurityDescriptor(
                SmpApiPortSecurityDescriptor as *mut _,
                1u8,
                core::ptr::null_mut(),
                0u8,
            );
        }

        if SmpProtectionMode & 1 != 0 {
            ProtectionRequired = 1u8;
        }

        if !(InitialCall != 0 || ProtectionRequired != 0) {
            return STATUS_SUCCESS;
        }

        Status = RtlAllocateAndInitializeSid(
            &mut WorldAuthority,
            1,
            SECURITY_WORLD_RID as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut WorldSid,
        );
        if !NT_SUCCESS(Status) {
            WorldSid = core::ptr::null_mut();
            break 'quickie;
        }

        Status = RtlAllocateAndInitializeSid(
            &mut NtAuthority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as ULONG,
            DOMAIN_ALIAS_RID_ADMINS as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut AdminSid,
        );
        if !NT_SUCCESS(Status) {
            AdminSid = core::ptr::null_mut();
            break 'quickie;
        }

        Status = RtlAllocateAndInitializeSid(
            &mut CreatorAuthority,
            1,
            SECURITY_CREATOR_OWNER_RID as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut OwnerSid,
        );
        if !NT_SUCCESS(Status) {
            OwnerSid = core::ptr::null_mut();
            break 'quickie;
        }

        Status = RtlAllocateAndInitializeSid(
            &mut NtAuthority,
            1,
            SECURITY_RESTRICTED_CODE_RID as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut RestrictedSid,
        );
        if !NT_SUCCESS(Status) {
            RestrictedSid = core::ptr::null_mut();
            break 'quickie;
        }

        Status = RtlAllocateAndInitializeSid(
            &mut NtAuthority,
            1,
            SECURITY_LOCAL_SYSTEM_RID as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut SystemSid,
        );
        if !NT_SUCCESS(Status) {
            SystemSid = core::ptr::null_mut();
            break 'quickie;
        }

        'not_initial: loop {
            if InitialCall == 0 {
                SidLength =
                    RtlLengthSid(WorldSid) + RtlLengthSid(RestrictedSid) + RtlLengthSid(AdminSid);
                SidLength *= 2;
                break 'not_initial;
            }

            SidLength = RtlLengthSid(SystemSid) + RtlLengthSid(AdminSid);
            AclLength = core::mem::size_of::<ACL>() as ULONG
                + 2 * core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
                + SidLength;

            'next_acl: loop {
                Acl = RtlAllocateHeap(RtlGetProcessHeap(), 0, AclLength as _) as PACL;
                if Acl.is_null() {
                    Status = STATUS_NO_MEMORY;
                }
                if !NT_SUCCESS(Status) {
                    break 'next_acl;
                }

                RtlCreateAcl(Acl, AclLength, ACL_REVISION2 as ULONG);
                RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, AdminSid);
                RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, SystemSid);

                RtlSetDaclSecurityDescriptor(SmpApiPortSecurityDescriptor as *mut _, 1u8, Acl, 0u8);
                break 'next_acl;
            }

            SidLength =
                RtlLengthSid(WorldSid) + RtlLengthSid(RestrictedSid) + RtlLengthSid(AdminSid);
            SidLength *= 2;
            AclLength = core::mem::size_of::<ACL>() as ULONG
                + 6 * core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
                + SidLength;
            Acl = RtlAllocateHeap(RtlGetProcessHeap(), 0, AclLength as _) as PACL;
            if Acl.is_null() {
                Status = STATUS_NO_MEMORY;
            }
            if !NT_SUCCESS(Status) {
                break 'not_initial;
            }

            RtlCreateAcl(Acl, AclLength, ACL_REVISION2 as ULONG);
            RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_EXECUTE, WorldSid);
            RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_EXECUTE, RestrictedSid);
            RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, AdminSid);
            RtlAddAccessAllowedAce(
                Acl,
                ACL_REVISION2 as ULONG,
                GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE,
                WorldSid,
            );
            RtlAddAccessAllowedAce(
                Acl,
                ACL_REVISION2 as ULONG,
                GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE,
                RestrictedSid,
            );
            RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, AdminSid);

            RtlGetAce(Acl, 3, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
            (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
            RtlGetAce(Acl, 4, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
            (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
            RtlGetAce(Acl, 5, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
            (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;

            RtlSetDaclSecurityDescriptor(SmpKnownDllsSecurityDescriptor as *mut _, 1u8, Acl, 0u8);
            break 'not_initial;
        }

        if ProtectionRequired == 0 {
            break 'quickie;
        }

        SidLength += RtlLengthSid(OwnerSid);
        AclLength = core::mem::size_of::<ACL>() as ULONG
            + 7 * core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + 2 * SidLength;
        Acl = RtlAllocateHeap(RtlGetProcessHeap(), 0, AclLength as _) as PACL;
        if Acl.is_null() {
            Status = STATUS_NO_MEMORY;
        }
        if !NT_SUCCESS(Status) {
            break 'quickie;
        }

        RtlCreateAcl(Acl, AclLength, ACL_REVISION2 as ULONG);
        RtlAddAccessAllowedAce(
            Acl,
            ACL_REVISION2 as ULONG,
            GENERIC_EXECUTE | GENERIC_READ,
            WorldSid,
        );
        RtlAddAccessAllowedAce(
            Acl,
            ACL_REVISION2 as ULONG,
            GENERIC_EXECUTE | GENERIC_READ,
            RestrictedSid,
        );
        RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, AdminSid);
        RtlAddAccessAllowedAce(
            Acl,
            ACL_REVISION2 as ULONG,
            GENERIC_EXECUTE | GENERIC_READ,
            WorldSid,
        );
        RtlAddAccessAllowedAce(
            Acl,
            ACL_REVISION2 as ULONG,
            GENERIC_EXECUTE | GENERIC_READ,
            RestrictedSid,
        );
        RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, AdminSid);
        RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, OwnerSid);

        RtlGetAce(Acl, 3, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
        (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
        RtlGetAce(Acl, 4, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
        (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
        RtlGetAce(Acl, 5, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
        (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
        RtlGetAce(Acl, 6, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
        (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;

        RtlSetDaclSecurityDescriptor(SmpPrimarySecurityDescriptor as *mut _, 1u8, Acl, 0u8);

        AclLength = core::mem::size_of::<ACL>() as ULONG
            + 7 * core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + 2 * SidLength;
        Acl = RtlAllocateHeap(RtlGetProcessHeap(), 0, AclLength as _) as PACL;
        if Acl.is_null() {
            Status = STATUS_NO_MEMORY;
        }
        if !NT_SUCCESS(Status) {
            break 'quickie;
        }

        RtlCreateAcl(Acl, AclLength, ACL_REVISION2 as ULONG);
        RtlAddAccessAllowedAce(
            Acl,
            ACL_REVISION2 as ULONG,
            GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE,
            WorldSid,
        );
        RtlAddAccessAllowedAce(
            Acl,
            ACL_REVISION2 as ULONG,
            GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE,
            RestrictedSid,
        );
        RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, AdminSid);
        RtlAddAccessAllowedAce(
            Acl,
            ACL_REVISION2 as ULONG,
            GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE,
            WorldSid,
        );
        RtlAddAccessAllowedAce(
            Acl,
            ACL_REVISION2 as ULONG,
            GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE,
            RestrictedSid,
        );
        RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, AdminSid);
        RtlAddAccessAllowedAce(Acl, ACL_REVISION2 as ULONG, GENERIC_ALL, OwnerSid);

        RtlGetAce(Acl, 3, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
        (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
        RtlGetAce(Acl, 4, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
        (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
        RtlGetAce(Acl, 5, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
        (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
        RtlGetAce(Acl, 6, &mut Ace as *mut _ as *mut *mut core::ffi::c_void);
        (*Ace).AceFlags = (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;

        RtlSetDaclSecurityDescriptor(SmpLiberalSecurityDescriptor as *mut _, 1u8, Acl, 0u8);
        break 'quickie;
    }

    if !OwnerSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, OwnerSid);
    }
    if !AdminSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AdminSid);
    }
    if !WorldSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, WorldSid);
    }
    if !SystemSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, SystemSid);
    }
    if !RestrictedSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, RestrictedSid);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpInitializeDosDevices() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut RegEntry: PSMP_REGISTRY_VALUE;
    let mut OldFlag: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut GlobalName: _UNICODE_STRING = core::mem::zeroed();
    let mut DirHandle: HANDLE = core::ptr::null_mut();
    let mut DosDevicesDirectory: HANDLE = core::ptr::null_mut();
    let mut NextEntry: PLIST_ENTRY;
    let Head: PLIST_ENTRY;

    let mut RegEntryVar: PSMP_REGISTRY_VALUE;

    RtlInitUnicodeString(&mut GlobalName, wstr!("\\??\0"));

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT) as u32;
    ObjectAttributes.ObjectName = &mut GlobalName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenDirectoryObject(
        core::ptr::addr_of_mut!(DosDevicesDirectory),
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    SmpDosDevicesObjectDirectory = DosDevicesDirectory;

    Head = core::ptr::addr_of_mut!(SmpDosDevicesList);
    while !IsListEmpty(Head) {
        NextEntry = RemoveHeadList(Head);
        RegEntryVar = ((NextEntry as usize) - (core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry)))
            as PSMP_REGISTRY_VALUE;
        RegEntry = RegEntryVar;

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = SmpDosDevicesObjectDirectory;
        ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT) as u32;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!((*RegEntry).Name);
        ObjectAttributes.SecurityDescriptor =
            SmpPrimarySecurityDescriptor as *mut core::ffi::c_void;
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        if !SmpPrimarySecurityDescriptor.is_null() {
            OldFlag = (*SmpPrimarySecurityDescriptor).Control;
            (*SmpPrimarySecurityDescriptor).Control |=
                SE_DACL_DEFAULTED as SECURITY_DESCRIPTOR_CONTROL;
        }

        Status = NtCreateSymbolicLinkObject(
            &mut DirHandle,
            SYMBOLIC_LINK_ALL_ACCESS,
            &mut ObjectAttributes,
            core::ptr::addr_of_mut!((*RegEntry).Value),
        );
        if Status == STATUS_OBJECT_NAME_EXISTS {
            NtMakeTemporaryObject(DirHandle);
            NtClose(DirHandle);

            Status = STATUS_SUCCESS;
            if (*RegEntry).Value.Length != 0 {
                ObjectAttributes.Attributes &= !(OBJ_OPENIF as u32);
                Status = NtCreateSymbolicLinkObject(
                    &mut DirHandle,
                    SYMBOLIC_LINK_ALL_ACCESS,
                    &mut ObjectAttributes,
                    core::ptr::addr_of_mut!((*RegEntry).Value),
                );
            }
        }

        if !ObjectAttributes.SecurityDescriptor.is_null() {
            (*SmpPrimarySecurityDescriptor).Control = OldFlag;
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        NtClose(DirHandle);

        if !(*RegEntry).AnsiValue.is_null() {
            RtlFreeHeap(
                RtlGetProcessHeap(),
                0,
                (*RegEntry).AnsiValue as *mut core::ffi::c_void,
            );
        }
        if !(*RegEntry).Value.Buffer.is_null() {
            RtlFreeHeap(
                RtlGetProcessHeap(),
                0,
                (*RegEntry).Value.Buffer as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpProcessModuleImports(
    _Unused: *mut core::ffi::c_void,
    ImportName: PCHAR,
) {
    let mut Length: ULONG = 0;
    let mut Buffer: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let DllName: PWCHAR;
    let DllValue: PWCHAR;
    let mut ImportString: ANSI_STRING = core::mem::zeroed();
    let mut ImportUnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    /* Skip NTDLL since it's already always mapped */
    if _stricmp(ImportName, b"ntdll.dll\0".as_ptr() as PCHAR) == 0 {
        return;
    }

    /* Initialize our strings */
    RtlInitAnsiString(&mut ImportString, ImportName);
    RtlInitEmptyUnicodeString(
        &mut ImportUnicodeString,
        Buffer.as_mut_ptr(),
        core::mem::size_of_val(&Buffer) as u16,
    );
    Status = RtlAnsiStringToUnicodeString(&mut ImportUnicodeString, &mut ImportString, 0u8);
    if !NT_SUCCESS(Status) {
        return;
    }

    /* Loop to find the DLL file extension */
    while Length < ImportUnicodeString.Length as ULONG {
        if *ImportUnicodeString
            .Buffer
            .add((Length / core::mem::size_of::<u16>() as ULONG) as usize)
            == '.' as u16
        {
            break;
        }
        Length += core::mem::size_of::<u16>() as ULONG;
    }

    /*
     * Break up the values as needed; the buffer acquires the form:
     * "dll_name.dll\0dll_name\0"
     */
    DllValue = ImportUnicodeString.Buffer;
    DllName = ImportUnicodeString.Buffer.add(
        ((ImportUnicodeString.Length as usize + core::mem::size_of::<u16>())
            / core::mem::size_of::<u16>()) as usize,
    );
    RtlStringCbCopyNW(
        DllName,
        ImportUnicodeString.MaximumLength as usize
            - (ImportUnicodeString.Length as usize + core::mem::size_of::<u16>()),
        ImportUnicodeString.Buffer,
        Length as usize,
    );

    /* Add the DLL to the list */
    SmpSaveRegistryValue(
        core::ptr::addr_of_mut!(SmpKnownDllsList),
        DllName,
        DllValue,
        1u8,
    );
}

unsafe extern "system" fn SmpInitializeKnownDllsInternal(
    Directory: PUNICODE_STRING,
    Path: PUNICODE_STRING,
) -> NTSTATUS {
    let mut DirFileHandle: HANDLE;
    let mut DirHandle: HANDLE;
    let mut SectionHandle: HANDLE = core::ptr::null_mut();
    let mut FileHandle: HANDLE;
    let mut LinkHandle: HANDLE = core::ptr::null_mut();
    let mut NtPath: _UNICODE_STRING = core::mem::zeroed();
    let mut SymLinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Status1: NTSTATUS;
    let mut NextEntry: PLIST_ENTRY;
    let mut RegEntry: PSMP_REGISTRY_VALUE;
    let mut ErrorParameters: [ULONG_PTR; 3] = [0; 3];
    let mut ErrorResponse: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut OldFlag: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut ImageCharacteristics: USHORT = 0;

    DirFileHandle = core::ptr::null_mut();
    DirHandle = core::ptr::null_mut();
    NtPath.Buffer = core::ptr::null_mut();

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT) as ULONG;
    ObjectAttributes.ObjectName = Directory;
    ObjectAttributes.SecurityDescriptor = SmpKnownDllsSecurityDescriptor as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateDirectoryObject(&mut DirHandle, DIRECTORY_ALL_ACCESS, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = 'quickie: {
        if RtlDosPathNameToNtPathName_U(
            (*Path).Buffer,
            &mut NtPath,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            break 'quickie STATUS_OBJECT_NAME_INVALID;
        }

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
        ObjectAttributes.ObjectName = &mut NtPath;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            &mut DirFileHandle,
            FILE_LIST_DIRECTORY | SYNCHRONIZE,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
        );
        if !NT_SUCCESS(Status) {
            break 'quickie Status;
        }

        if !SmpPrimarySecurityDescriptor.is_null() {
            OldFlag = (*SmpPrimarySecurityDescriptor).Control;
            (*SmpPrimarySecurityDescriptor).Control |=
                SE_DACL_DEFAULTED as SECURITY_DESCRIPTOR_CONTROL;
        }

        RtlInitUnicodeString(
            &mut SymLinkName,
            w!("KnownDllPath").as_ptr() as PWSTR,
        );

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = DirHandle;
        ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT) as ULONG;
        ObjectAttributes.ObjectName = &mut SymLinkName;
        ObjectAttributes.SecurityDescriptor = SmpPrimarySecurityDescriptor as PVOID;
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtCreateSymbolicLinkObject(
            &mut LinkHandle,
            SYMBOLIC_LINK_ALL_ACCESS,
            &mut ObjectAttributes,
            Path,
        );

        if !SmpPrimarySecurityDescriptor.is_null() {
            (*SmpPrimarySecurityDescriptor).Control = OldFlag;
        }

        if !NT_SUCCESS(Status) {
            break 'quickie Status;
        }

        Status1 = NtClose(LinkHandle);
        let _ = Status1;

        NextEntry = SmpKnownDllsList.Flink;
        while NextEntry != &raw mut SmpKnownDllsList {
            RegEntry = (NextEntry as *mut u8)
                .offset(-(core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry) as isize))
                as PSMP_REGISTRY_VALUE;
            NextEntry = (*NextEntry).Flink;

            if !SmpFindRegistryValue(&raw mut SmpExcludeKnownDllsList, (*RegEntry).Name.Buffer)
                .is_null()
                || !SmpFindRegistryValue(&raw mut SmpExcludeKnownDllsList, (*RegEntry).Value.Buffer)
                    .is_null()
            {
                continue;
            }

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = DirFileHandle;
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
            ObjectAttributes.ObjectName = &mut (*RegEntry).Value;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            FileHandle = core::ptr::null_mut();
            Status1 = NtOpenFile(
                &mut FileHandle,
                SYNCHRONIZE | FILE_EXECUTE,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                FILE_SHARE_READ | FILE_SHARE_DELETE,
                FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
            );
            if !NT_SUCCESS(Status1) {
                continue;
            }

            Status = LdrVerifyImageMatchesChecksum(
                ((FileHandle as ULONG_PTR) | 1) as HANDLE,
                core::mem::transmute(SmpProcessModuleImports as usize),
                RegEntry as PVOID,
                &mut ImageCharacteristics,
            );
            if !NT_SUCCESS(Status) {
                RtlInitUnicodeString(&mut ErrorResponse, w!("Verification of a KnownDLL failed.").as_ptr() as PWSTR);
                ErrorParameters[0] = &mut ErrorResponse as *mut _ as ULONG_PTR;
                ErrorParameters[1] = Status as ULONG_PTR;
                ErrorParameters[2] = &mut (*RegEntry).Value as *mut _ as ULONG_PTR;
                SmpTerminate(
                    ErrorParameters.as_mut_ptr(),
                    5,
                    ErrorParameters.len() as ULONG,
                );
            } else if (ImageCharacteristics & IMAGE_FILE_DLL as USHORT) == 0 {
                RtlInitUnicodeString(&mut ErrorResponse, w!("Non-DLL file included in KnownDLL list.").as_ptr() as PWSTR);
                ErrorParameters[0] = &mut ErrorResponse as *mut _ as ULONG_PTR;
                ErrorParameters[1] = STATUS_INVALID_IMPORT_OF_NON_DLL as ULONG_PTR;
                ErrorParameters[2] = &mut (*RegEntry).Value as *mut _ as ULONG_PTR;
                SmpTerminate(
                    ErrorParameters.as_mut_ptr(),
                    5,
                    ErrorParameters.len() as ULONG,
                );
            }

            if !SmpLiberalSecurityDescriptor.is_null() {
                OldFlag = (*SmpLiberalSecurityDescriptor).Control;
                (*SmpLiberalSecurityDescriptor).Control |=
                    SE_DACL_DEFAULTED as SECURITY_DESCRIPTOR_CONTROL;
            }

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = DirHandle;
            ObjectAttributes.Attributes = OBJ_PERMANENT as ULONG;
            ObjectAttributes.ObjectName = &mut (*RegEntry).Value;
            ObjectAttributes.SecurityDescriptor = SmpLiberalSecurityDescriptor as PVOID;
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtCreateSection(
                &mut SectionHandle,
                SECTION_ALL_ACCESS,
                &mut ObjectAttributes,
                core::ptr::null_mut(),
                PAGE_EXECUTE,
                SEC_IMAGE,
                FileHandle,
            );

            if !SmpLiberalSecurityDescriptor.is_null() {
                (*SmpLiberalSecurityDescriptor).Control = OldFlag;
            }

            if NT_SUCCESS(Status) {
                Status1 = NtClose(SectionHandle);
                let _ = Status1;
            }

            Status1 = NtClose(FileHandle);
            let _ = Status1;
        }

        Status
    };

    if !DirHandle.is_null() {
        Status1 = NtClose(DirHandle);
        let _ = Status1;
    }
    if !DirFileHandle.is_null() {
        Status1 = NtClose(DirFileHandle);
        let _ = Status1;
    }
    if !NtPath.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPath.Buffer as PVOID);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpInitializeKnownDlls() -> NTSTATUS {
    let Status: NTSTATUS;
    let mut RegEntry: PSMP_REGISTRY_VALUE;
    let mut KnownDllsName: _UNICODE_STRING = core::mem::zeroed();
    let Head: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;

    RtlInitUnicodeString(&mut KnownDllsName, wstr!("\\KnownDlls\0"));
    Status = SmpInitializeKnownDllsInternal(&mut KnownDllsName, &raw mut SmpKnownDllPath);

    Head = core::ptr::addr_of_mut!(SmpKnownDllsList);
    while !IsListEmpty(Head) {
        NextEntry = RemoveHeadList(Head);

        RegEntry = ((NextEntry as usize) - core::mem::offset_of!(_SMP_REGISTRY_VALUE, Entry))
            as PSMP_REGISTRY_VALUE;
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*RegEntry).AnsiValue as PVOID);
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*RegEntry).Value.Buffer as PVOID);
        RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpCreateDynamicEnvironmentVariables() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut BasicInfo: SYSTEM_BASIC_INFORMATION = core::mem::zeroed();
    let mut ProcessorInfo: SYSTEM_PROCESSOR_INFORMATION = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut DestinationString: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut KeyHandle2: HANDLE = core::ptr::null_mut();
    let mut ValueData: PWCHAR;
    let mut ResultLength: ULONG = 0;
    let mut StrLength: usize = 0;
    let mut ValueBuffer: [WCHAR; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + 512] =
        [0; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + 512];
    let mut ValueBuffer2: [WCHAR; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + 512] =
        [0; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + 512];
    let PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION =
        ValueBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
    let PartialInfo2: PKEY_VALUE_PARTIAL_INFORMATION =
        ValueBuffer2.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;

    Status = NtQuerySystemInformation(
        SystemBasicInformation,
        core::ptr::addr_of_mut!(BasicInfo) as *mut _,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = NtQuerySystemInformation(
        SystemProcessorInformation,
        core::ptr::addr_of_mut!(ProcessorInfo) as *mut _,
        core::mem::size_of::<SYSTEM_PROCESSOR_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(
        &mut DestinationString,
        wstr!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Session Manager\\Environment\0"),
    );
    InitializeObjectAttributes_inline(
        &mut ObjectAttributes,
        &mut DestinationString,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(&mut KeyHandle, GENERIC_WRITE, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(&mut ValueName, wstr!("OS\0"));
    ValueData = wstr!("Windows_NT\0");
    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        ValueData as *mut _,
        ((wcslen(ValueData) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        return Status;
    }

    RtlInitUnicodeString(&mut ValueName, wstr!("PROCESSOR_ARCHITECTURE\0"));
    match ProcessorInfo.ProcessorArchitecture as u32 {
        x if x == PROCESSOR_ARCHITECTURE_INTEL as u32 => {
            ValueData = wstr!("x86\0");
        }
        x if x == PROCESSOR_ARCHITECTURE_AMD64 as u32 => {
            ValueData = wstr!("AMD64\0");
        }
        x if x == PROCESSOR_ARCHITECTURE_IA64 as u32 => {
            ValueData = wstr!("IA64\0");
        }
        _ => {
            ValueData = wstr!("Unknown\0");
        }
    }

    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        ValueData as *mut _,
        ((wcslen(ValueData) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        return Status;
    }

    RtlInitUnicodeString(&mut ValueName, wstr!("PROCESSOR_LEVEL\0"));
    swprintf(
        ValueBuffer.as_mut_ptr(),
        wstr!("%u\0"),
        ProcessorInfo.ProcessorLevel as u32,
    );
    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        ValueBuffer.as_mut_ptr() as *mut _,
        ((wcslen(ValueBuffer.as_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        return Status;
    }

    RtlInitUnicodeString(
        &mut DestinationString,
        wstr!("\\Registry\\Machine\\Hardware\\Description\\System\\CentralProcessor\\0\0"),
    );
    InitializeObjectAttributes_inline(
        &mut ObjectAttributes,
        &mut DestinationString,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(&mut KeyHandle2, KEY_READ, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        return Status;
    }

    RtlInitUnicodeString(&mut ValueName, wstr!("Identifier\0"));
    Status = NtQueryValueKey(
        KeyHandle2,
        &mut ValueName,
        KeyValuePartialInformation,
        PartialInfo as *mut _,
        core::mem::size_of_val(&ValueBuffer) as ULONG,
        &mut ResultLength,
    );
    if !NT_SUCCESS(Status)
        || ((*PartialInfo).Type != REG_SZ && (*PartialInfo).Type != REG_EXPAND_SZ)
    {
        NtClose(KeyHandle2);
        NtClose(KeyHandle);
        return Status;
    }

    let data_offset = {
        let base = PartialInfo as usize;
        let dataptr = core::ptr::addr_of!((*PartialInfo).Data) as usize;
        dataptr - base
    };
    RtlInitEmptyUnicodeString(
        &mut DestinationString,
        core::ptr::addr_of_mut!((*PartialInfo).Data) as PWCHAR,
        (core::mem::size_of_val(&ValueBuffer) - data_offset) as USHORT,
    );
    RtlStringCbLengthW(
        DestinationString.Buffer,
        (*PartialInfo).DataLength as usize,
        &mut StrLength,
    );
    DestinationString.Length = StrLength as USHORT;

    RtlInitUnicodeString(&mut ValueName, wstr!("VendorIdentifier\0"));
    Status = NtQueryValueKey(
        KeyHandle2,
        &mut ValueName,
        KeyValuePartialInformation,
        PartialInfo2 as *mut _,
        core::mem::size_of_val(&ValueBuffer2) as ULONG,
        &mut ResultLength,
    );
    NtClose(KeyHandle2);
    if NT_SUCCESS(Status)
        && ((*PartialInfo2).Type == REG_SZ || (*PartialInfo2).Type == REG_EXPAND_SZ)
    {
        // RtlStringCbPrintfW is an ntstrsafe inline (no export). Use the
        // exported variadic _snwprintf: count is in WCHARs (not bytes) and the
        // %.*s precision argument is a c_int.
        _snwprintf(
            DestinationString
                .Buffer
                .add(DestinationString.Length as usize / core::mem::size_of::<WCHAR>()),
            (DestinationString.MaximumLength - DestinationString.Length) as usize
                / core::mem::size_of::<WCHAR>(),
            wstr!(", %.*s\0"),
            ((*PartialInfo2).DataLength as usize / core::mem::size_of::<WCHAR>())
                as core::ffi::c_int,
            core::ptr::addr_of!((*PartialInfo2).Data) as PWCHAR,
        );
        DestinationString.Length =
            (wcslen(DestinationString.Buffer) * core::mem::size_of::<WCHAR>()) as USHORT;
    }

    RtlInitUnicodeString(&mut ValueName, wstr!("PROCESSOR_IDENTIFIER\0"));
    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        DestinationString.Buffer as *mut _,
        DestinationString.Length as ULONG + core::mem::size_of::<WCHAR>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        return Status;
    }

    RtlInitUnicodeString(&mut ValueName, wstr!("PROCESSOR_REVISION\0"));
    match ProcessorInfo.ProcessorArchitecture as u32 {
        x if x == PROCESSOR_ARCHITECTURE_INTEL as u32 => {
            if (ProcessorInfo.ProcessorRevision >> 8) == 0xFF {
                swprintf(
                    ValueBuffer.as_mut_ptr(),
                    wstr!("%02x\0"),
                    (ProcessorInfo.ProcessorRevision & 0xFF) as u32,
                );
                _wcsupr(ValueBuffer.as_mut_ptr());
            } else {
                swprintf(
                    ValueBuffer.as_mut_ptr(),
                    wstr!("%04x\0"),
                    ProcessorInfo.ProcessorRevision as u32,
                );
            }
        }
        x if x == PROCESSOR_ARCHITECTURE_IA64 as u32
            || x == PROCESSOR_ARCHITECTURE_AMD64 as u32 =>
        {
            swprintf(
                ValueBuffer.as_mut_ptr(),
                wstr!("%04x\0"),
                ProcessorInfo.ProcessorRevision as u32,
            );
        }
        _ => {
            swprintf(
                ValueBuffer.as_mut_ptr(),
                wstr!("%u\0"),
                ProcessorInfo.ProcessorRevision as u32,
            );
        }
    }

    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        ValueBuffer.as_mut_ptr() as *mut _,
        ((wcslen(ValueBuffer.as_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        return Status;
    }

    RtlInitUnicodeString(&mut ValueName, wstr!("NUMBER_OF_PROCESSORS\0"));
    swprintf(
        ValueBuffer.as_mut_ptr(),
        wstr!("%d\0"),
        BasicInfo.NumberOfProcessors as i32,
    );
    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_SZ,
        ValueBuffer.as_mut_ptr() as *mut _,
        ((wcslen(ValueBuffer.as_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        return Status;
    }

    RtlInitUnicodeString(
        &mut DestinationString,
        wstr!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Safeboot\\Option\0"),
    );
    InitializeObjectAttributes_inline(
        &mut ObjectAttributes,
        &mut DestinationString,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(&mut KeyHandle2, KEY_ALL_ACCESS, &mut ObjectAttributes);
    if NT_SUCCESS(Status) {
        RtlInitUnicodeString(&mut ValueName, wstr!("OptionValue\0"));
        Status = NtQueryValueKey(
            KeyHandle2,
            &mut ValueName,
            KeyValuePartialInformation,
            PartialInfo as *mut _,
            core::mem::size_of_val(&ValueBuffer) as ULONG,
            &mut ResultLength,
        );
        NtClose(KeyHandle2);
        if NT_SUCCESS(Status)
            && (*PartialInfo).Type == REG_DWORD
            && ((*PartialInfo).DataLength as usize >= core::mem::size_of::<ULONG>())
        {
            RtlInitUnicodeString(&mut ValueName, wstr!("SAFEBOOT_OPTION\0"));
            let optval = *(core::ptr::addr_of!((*PartialInfo).Data) as *const ULONG);
            match optval {
                1 => {
                    wcscpy(ValueBuffer.as_mut_ptr(), wstr!("MINIMAL\0"));
                }
                2 => {
                    wcscpy(ValueBuffer.as_mut_ptr(), wstr!("NETWORK\0"));
                }
                3 => {
                    wcscpy(ValueBuffer.as_mut_ptr(), wstr!("DSREPAIR\0"));
                }
                _ => {}
            }

            Status = NtSetValueKey(
                KeyHandle,
                &mut ValueName,
                0,
                REG_SZ,
                ValueBuffer.as_mut_ptr() as *mut _,
                ((wcslen(ValueBuffer.as_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as ULONG,
            );
            if !NT_SUCCESS(Status) {
                NtClose(KeyHandle);
                return Status;
            }
        }
    }

    NtClose(KeyHandle);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SmpProcessFileRenames() -> NTSTATUS {
    let mut OldState: BOOLEAN = 0;
    let mut HavePrivilege: BOOLEAN = 0u8;
    let mut Status: NTSTATUS;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut OtherFileHandle: HANDLE = core::ptr::null_mut();
    let mut InformationClass: FILE_INFORMATION_CLASS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileString: _UNICODE_STRING = core::mem::zeroed();
    let mut BasicInfo: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut DeleteInformation: FILE_DISPOSITION_INFORMATION = core::mem::zeroed();
    let mut Buffer: PFILE_RENAME_INFORMATION;
    let Head: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut RegEntry: PSMP_REGISTRY_VALUE;
    let mut FileName: PWCHAR;
    let mut ValueLength: ULONG;
    let mut Length: ULONG;

    Status = RtlAdjustPrivilege(SE_RESTORE_PRIVILEGE, 1u8, 0u8, &mut OldState);
    if NT_SUCCESS(Status) {
        HavePrivilege = 1u8;
    }

    Head = core::ptr::addr_of_mut!(SmpFileRenameList);
    while !IsListEmpty(Head) {
        NextEntry = RemoveHeadList(Head);
        RegEntry = (NextEntry as usize - core::mem::offset_of!(_SMP_REGISTRY_VALUE, Entry))
            as PSMP_REGISTRY_VALUE;

        if (*RegEntry).Value.Length == 0 && *(*RegEntry).Name.Buffer == ('@' as u16) {
            (*RegEntry).Name.Length -= core::mem::size_of::<u16>() as u16;
            (*RegEntry).Name.Buffer = (*RegEntry).Name.Buffer.add(1);
        }

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!((*RegEntry).Name);
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            &mut OtherFileHandle,
            DELETE | SYNCHRONIZE,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_SYNCHRONOUS_IO_NONALERT,
        );

        'quickie: {
            if !NT_SUCCESS(Status) {
                break 'quickie;
            }

            ValueLength = (*RegEntry).Value.Length as ULONG;
            FileName = core::ptr::null_mut();
            if ValueLength == 0 {
                InformationClass = FileDispositionInformation;
                Length = core::mem::size_of::<FILE_DISPOSITION_INFORMATION>() as ULONG;
                Buffer = core::ptr::addr_of_mut!(DeleteInformation) as PFILE_RENAME_INFORMATION;
                DeleteInformation.DeleteFileA = 1u8;
            } else {
                InformationClass = FileRenameInformation;
                Length = ValueLength + core::mem::size_of::<FILE_RENAME_INFORMATION>() as ULONG;

                FileName = (*RegEntry).Value.Buffer;
                if *FileName == ('!' as u16) || *FileName == ('@' as u16) {
                    FileName = FileName.add(1);
                    Length -= core::mem::size_of::<u16>() as ULONG;
                }

                Buffer = RtlAllocateHeap(RtlGetProcessHeap(), SmBaseTag, Length as _)
                    as PFILE_RENAME_INFORMATION;
                if !Buffer.is_null() {
                    (*Buffer).RootDirectory = core::ptr::null_mut();
                    (*Buffer).FileNameLength =
                        Length - core::mem::size_of::<FILE_RENAME_INFORMATION>() as ULONG;
                    (*Buffer).ReplaceIfExists = if FileName != (*RegEntry).Value.Buffer {
                        1u8
                    } else {
                        0u8
                    };
                    core::ptr::copy_nonoverlapping(
                        FileName as *const u8,
                        (*Buffer).FileName.as_mut_ptr() as *mut u8,
                        (*Buffer).FileNameLength as usize,
                    );
                } else {
                    Status = STATUS_NO_MEMORY;
                }
            }

            if NT_SUCCESS(Status) {
                Status = NtSetInformationFile(
                    OtherFileHandle,
                    &mut IoStatusBlock,
                    Buffer as PVOID,
                    Length,
                    InformationClass,
                );

                if !NT_SUCCESS(Status)
                    && InformationClass == FileRenameInformation
                    && Status == STATUS_OBJECT_NAME_COLLISION
                    && (*Buffer).ReplaceIfExists != 0
                {
                    FileString.Length =
                        (*RegEntry).Value.Length - core::mem::size_of::<u16>() as u16;
                    FileString.MaximumLength =
                        (*RegEntry).Value.MaximumLength - core::mem::size_of::<u16>() as u16;
                    FileString.Buffer = FileName;
                    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
                    ObjectAttributes.RootDirectory = core::ptr::null_mut();
                    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
                    ObjectAttributes.ObjectName = &mut FileString;
                    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
                    Status = NtOpenFile(
                        &mut FileHandle,
                        FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
                        &mut ObjectAttributes,
                        &mut IoStatusBlock,
                        FILE_SHARE_READ | FILE_SHARE_WRITE,
                        FILE_SYNCHRONOUS_IO_NONALERT,
                    );
                    if NT_SUCCESS(Status) {
                        core::ptr::write_bytes(
                            &mut BasicInfo as *mut FILE_BASIC_INFORMATION as *mut u8,
                            0,
                            core::mem::size_of::<FILE_BASIC_INFORMATION>(),
                        );
                        BasicInfo.FileAttributes = FILE_ATTRIBUTE_NORMAL;
                        Status = NtSetInformationFile(
                            FileHandle,
                            &mut IoStatusBlock,
                            &mut BasicInfo as *mut FILE_BASIC_INFORMATION as PVOID,
                            core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
                            FileBasicInformation,
                        );
                        NtClose(FileHandle);
                        if NT_SUCCESS(Status) {
                            Status = NtSetInformationFile(
                                OtherFileHandle,
                                &mut IoStatusBlock,
                                Buffer as PVOID,
                                Length,
                                FileRenameInformation,
                            );
                        }
                    }
                }
            }

            NtClose(OtherFileHandle);
        }

        if !(*RegEntry).AnsiValue.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*RegEntry).AnsiValue as PVOID);
        }
        if !(*RegEntry).Value.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*RegEntry).Value.Buffer as PVOID);
        }
        RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry as PVOID);
    }

    if HavePrivilege != 0 {
        RtlAdjustPrivilege(SE_RESTORE_PRIVILEGE, 0u8, 0u8, &mut OldState);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpLoadDataFromRegistry(InitialCommand: PUNICODE_STRING) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Head: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut RegEntry: PSMP_REGISTRY_VALUE;
    let OriginalEnvironment: PWSTR;
    let mut MuSessionId: ULONG = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut DestinationString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&raw mut SmpDebugKeyword, wstr!("debug\0"));
    RtlInitUnicodeString(&raw mut SmpASyncKeyword, wstr!("async\0"));
    RtlInitUnicodeString(&raw mut SmpAutoChkKeyword, wstr!("autocheck\0"));

    InitializeListHead(core::ptr::addr_of_mut!(SmpBootExecuteList));
    InitializeListHead(core::ptr::addr_of_mut!(SmpSetupExecuteList));
    InitializeListHead(core::ptr::addr_of_mut!(SmpPagingFileList));
    InitializeListHead(core::ptr::addr_of_mut!(SmpDosDevicesList));
    InitializeListHead(core::ptr::addr_of_mut!(SmpFileRenameList));
    InitializeListHead(core::ptr::addr_of_mut!(SmpKnownDllsList));
    InitializeListHead(core::ptr::addr_of_mut!(SmpExcludeKnownDllsList));
    InitializeListHead(core::ptr::addr_of_mut!(SmpSubSystemList));
    InitializeListHead(core::ptr::addr_of_mut!(SmpSubSystemsToLoad));
    InitializeListHead(core::ptr::addr_of_mut!(SmpSubSystemsToDefer));
    InitializeListHead(core::ptr::addr_of_mut!(SmpExecuteList));

    SmpPagingFileInitialize();

    Status = RtlCreateEnvironment(1u8, &raw mut SmpDefaultEnvironment as *mut PWCHAR);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(
        &mut DestinationString,
        wstr!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\MiniNT\0"),
    );
    InitializeObjectAttributes_inline(
        &mut ObjectAttributes,
        &mut DestinationString,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(&mut KeyHandle, KEY_ALL_ACCESS, &mut ObjectAttributes);
    if NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        MiniNTBoot = 1u8;
    }

    RtlInitUnicodeString(
        &mut DestinationString,
        wstr!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Session Manager\\Environment\0"),
    );
    InitializeObjectAttributes_inline(
        &mut ObjectAttributes,
        &mut DestinationString,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(&mut KeyHandle, KEY_ALL_ACCESS, &mut ObjectAttributes);
    if NT_SUCCESS(Status) {
        RtlInitUnicodeString(&mut DestinationString, wstr!("SAFEBOOT_OPTION\0"));
        NtDeleteValueKey(KeyHandle, &mut DestinationString);
        NtClose(KeyHandle);
    }

    let Peb = (*(NtCurrentTeb())).ProcessEnvironmentBlock;
    OriginalEnvironment = (*(*Peb).ProcessParameters).Environment;
    (*(*Peb).ProcessParameters).Environment = SmpDefaultEnvironment;
    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_CONTROL,
        wstr!("Session Manager\0"),
        SmpRegistryConfigurationTable.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    SmpDefaultEnvironment = (*(*Peb).ProcessParameters).Environment as PWCHAR;
    (*(*Peb).ProcessParameters).Environment = OriginalEnvironment;
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = SmpInitializeDosDevices();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(&mut DestinationString, wstr!("\\Sessions\0"));
    InitializeObjectAttributes_inline(
        &mut ObjectAttributes,
        &mut DestinationString,
        OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT,
        core::ptr::null_mut(),
        SmpPrimarySecurityDescriptor as PVOID,
    );
    Status = NtCreateDirectoryObject(
        &raw mut SmpSessionsObjectDirectory,
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Head = core::ptr::addr_of_mut!(SmpBootExecuteList);
    while !IsListEmpty(Head) {
        NextEntry = RemoveHeadList(Head);

        RegEntry = (NextEntry as usize - core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry))
            as PSMP_REGISTRY_VALUE;
        SmpExecuteCommand(&mut (*RegEntry).Name, 0, core::ptr::null_mut(), 0);

        if !(*RegEntry).AnsiValue.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*RegEntry).AnsiValue as PVOID);
        }
        if !(*RegEntry).Value.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*RegEntry).Value.Buffer as PVOID);
        }
        RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry as PVOID);
    }

    if MiniNTBoot == 0 {
        SmpProcessFileRenames();
    }

    Status = SmpInitializeKnownDlls();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if MiniNTBoot == 0 {
        Head = core::ptr::addr_of_mut!(SmpPagingFileList);
        while !IsListEmpty(Head) {
            NextEntry = RemoveHeadList(Head);

            RegEntry = (NextEntry as usize - core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry))
                as PSMP_REGISTRY_VALUE;
            SmpCreatePagingFileDescriptor(&mut (*RegEntry).Name);

            if !(*RegEntry).AnsiValue.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*RegEntry).AnsiValue as PVOID);
            }
            if !(*RegEntry).Value.Buffer.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*RegEntry).Value.Buffer as PVOID);
            }
            RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry as PVOID);
        }

        SmpCreatePagingFiles();
    }

    NtInitializeRegistry(CM_BOOT_FLAG_SMSS as u16);

    Status = SmpCreateDynamicEnvironmentVariables();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = SmpLoadSubSystemsForMuSession(
        &mut MuSessionId,
        &raw mut SmpWindowsSubSysProcessId,
        InitialCommand,
    );
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpInit(
    InitialCommand: PUNICODE_STRING,
    ProcessHandle: PHANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Status2: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PortName: _UNICODE_STRING = core::mem::zeroed();
    let mut EventName: _UNICODE_STRING = core::mem::zeroed();
    let mut EventHandle: HANDLE = core::ptr::null_mut();
    let mut PortHandle: HANDLE = core::ptr::null_mut();
    let mut HardErrorMode: ULONG;

    /* Create the SMSS Heap */
    SmBaseTag = RtlCreateTagHeap(
        RtlGetProcessHeap(),
        0,
        b"S\0M\0S\0S\0!\0\0\0".as_ptr() as PWCHAR,
        b"I\0N\0I\0T\0\0\0".as_ptr() as PWCHAR,
    );
    SmpHeap = RtlGetProcessHeap();

    /* Enable hard errors */
    HardErrorMode = 1;
    NtSetInformationProcess(
        NtCurrentProcess(),
        ProcessDefaultHardErrorMode,
        core::ptr::addr_of_mut!(HardErrorMode) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    /* Initialize the subsystem list and the session list, plus their locks */
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
    InitializeListHead(core::ptr::addr_of_mut!(SmpKnownSubSysHead));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(SmpSessionListLock));
    InitializeListHead(core::ptr::addr_of_mut!(SmpSessionListHead));

    /* Initialize the process list */
    InitializeListHead(core::ptr::addr_of_mut!(NativeProcessList));

    /* Initialize session parameters */
    SmpNextSessionId = 1;
    SmpNextSessionIdScanMode = 0;
    SmpDbgSsLoaded = 0;

    /* Create the initial security descriptors */
    Status = SmpCreateSecurityDescriptors(1);
    if !NT_SUCCESS(Status) {
        /* Fail */
        SmpInitProgressByLine = line!();
        SmpInitReturnStatus = Status;
        SmpInitLastCall = SmpCreateSecurityDescriptors as PVOID;
        return Status;
    }

    /* Initialize subsystem names */
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(SmpSubsystemName),
        b"N\0T\0-\0S\0e\0s\0s\0i\0o\0n\0 \0M\0a\0n\0a\0g\0e\0r\0\0\0".as_ptr() as PWCHAR,
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PosixName),
        b"P\0O\0S\0I\0X\0\0\0".as_ptr() as PWCHAR,
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(Os2Name),
        b"O\0S\02\0\0\0".as_ptr() as PWCHAR,
    );

    /* Create the SM API Port */
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PortName),
        wstr!("\\SmApiPort\0"),
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(PortName);
    ObjectAttributes.SecurityDescriptor = SmpApiPortSecurityDescriptor as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtCreatePort(
        core::ptr::addr_of_mut!(PortHandle),
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::mem::size_of::<SB_CONNECTION_INFO>() as ULONG,
        core::mem::size_of::<SM_API_MSG>() as ULONG,
        (core::mem::size_of::<SB_API_MSG>() * 32) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    SmpDebugPort = PortHandle;

    /* Create two SM API threads */
    Status = RtlCreateUserThread(
        NtCurrentProcess(),
        core::ptr::null_mut(),
        0,
        0,
        0,
        0,
        Some(SmpApiLoop),
        PortHandle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    Status = RtlCreateUserThread(
        NtCurrentProcess(),
        core::ptr::null_mut(),
        0,
        0,
        0,
        0,
        Some(SmpApiLoop),
        PortHandle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Create the write event that autochk can set after running */
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(EventName),
        b"\\\0D\0e\0v\0i\0c\0e\0\\\0V\0o\0l\0u\0m\0e\0s\0S\0a\0f\0e\0F\0o\0r\0W\0r\0i\0t\0e\0A\0c\0c\0e\0s\0s\0\0\0".as_ptr() as PWCHAR,
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_PERMANENT;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(EventName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status2 = NtCreateEvent(
        core::ptr::addr_of_mut!(EventHandle),
        EVENT_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0,
        0,
    );
    if !NT_SUCCESS(Status2) { /* Should never really fail */ }

    /* Now initialize everything else based on the registry parameters */
    Status = SmpLoadDataFromRegistry(InitialCommand);
    if NT_SUCCESS(Status) {
        /* Autochk should've run now. Set the event and save the CSRSS handle */
        *ProcessHandle = SmpWindowsSubSysProcess;

        if (*ProcessHandle).is_null() {
            NtCreateEvent(
                ProcessHandle,
                EVENT_ALL_ACCESS,
                core::ptr::null_mut(),
                _EVENT_TYPE_NotificationEvent,
                0,
            );
        }

        NtSetEvent(EventHandle, core::ptr::null_mut());
        NtClose(EventHandle);
    }

    /* All done */
    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Windows-Compatible Session Manager
//  * LICENSE:         BSD 2-Clause License
//  * FILE:            base/system/smss/sminit.c
//  * PURPOSE:         Main SMSS Code
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "smss.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// UNICODE_STRING SmpSubsystemName, PosixName, Os2Name;
// LIST_ENTRY SmpBootExecuteList, SmpSetupExecuteList;
// LIST_ENTRY SmpPagingFileList, SmpDosDevicesList, SmpFileRenameList;
// LIST_ENTRY SmpKnownDllsList, SmpExcludeKnownDllsList;
// LIST_ENTRY SmpSubSystemList, SmpSubSystemsToLoad, SmpSubSystemsToDefer;
// LIST_ENTRY SmpExecuteList, NativeProcessList;
//
// PVOID SmpHeap;
// ULONG SmBaseTag;
// HANDLE SmpDebugPort, SmpDosDevicesObjectDirectory;
// PWCHAR SmpDefaultEnvironment, SmpDefaultLibPathBuffer;
// UNICODE_STRING SmpKnownDllPath, SmpDefaultLibPath;
// ULONG SmpCalledConfigEnv;
//
// ULONG SmpInitProgressByLine;
// NTSTATUS SmpInitReturnStatus;
// PVOID SmpInitLastCall;
//
// SECURITY_DESCRIPTOR SmpPrimarySDBody, SmpLiberalSDBody, SmpKnownDllsSDBody;
// SECURITY_DESCRIPTOR SmpApiPortSDBody;
// PISECURITY_DESCRIPTOR SmpPrimarySecurityDescriptor, SmpLiberalSecurityDescriptor;
// PISECURITY_DESCRIPTOR SmpKnownDllsSecurityDescriptor, SmpApiPortSecurityDescriptor;
//
// ULONG SmpAllowProtectedRenames, SmpProtectionMode = 1;
// BOOLEAN MiniNTBoot = FALSE;
//
// #define SMSS_CHECKPOINT(x, y)           \
// {                                       \
//     SmpInitProgressByLine = __LINE__;   \
//     SmpInitReturnStatus = (y);          \
//     SmpInitLastCall = (x);              \
// }
//
// /* REGISTRY CONFIGURATION *****************************************************/
//
// NTSTATUS
// NTAPI
// SmpSaveRegistryValue(IN PLIST_ENTRY ListAddress,
//                      IN PWSTR Name,
//                      IN PWCHAR Value,
//                      IN BOOLEAN Flags)
// {
//     PSMP_REGISTRY_VALUE RegEntry;
//     UNICODE_STRING NameString, ValueString;
//     ANSI_STRING AnsiValueString;
//     PLIST_ENTRY NextEntry;
//
//     /* Convert to unicode strings */
//     RtlInitUnicodeString(&NameString, Name);
//     RtlInitUnicodeString(&ValueString, Value);
//
//     /* In case this is the first value, initialize a new list/structure */
//     RegEntry = NULL;
//
//     /* Check if we should do a duplicate check */
//     if (Flags)
//     {
//         /* Loop the current list */
//         NextEntry = ListAddress->Flink;
//         while (NextEntry != ListAddress)
//         {
//             /* Get each entry */
//             RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//
//             /* Check if the value name matches */
//             if (!RtlCompareUnicodeString(&RegEntry->Name, &NameString, TRUE))
//             {
//                 /* Check if the value is the exact same thing */
//                 if (!RtlCompareUnicodeString(&RegEntry->Value, &ValueString, TRUE))
//                 {
//                     /* Fail -- the same setting is being set twice */
//                     return STATUS_OBJECT_NAME_EXISTS;
//                 }
//
//                 /* We found the list, and this isn't a duplicate value */
//                 break;
//             }
//
//             /* This wasn't a match, keep going */
//             NextEntry = NextEntry->Flink;
//             RegEntry = NULL;
//         }
//     }
//
//     /* Are we adding on, or creating a new entry */
//     if (!RegEntry)
//     {
//         /* A new entry -- allocate it */
//         RegEntry = RtlAllocateHeap(RtlGetProcessHeap(),
//                                    SmBaseTag,
//                                    sizeof(SMP_REGISTRY_VALUE) +
//                                    NameString.MaximumLength);
//         if (!RegEntry) return STATUS_NO_MEMORY;
//
//         /* Initialize the list and set all values to NULL */
//         InitializeListHead(&RegEntry->Entry);
//         RegEntry->AnsiValue = NULL;
//         RegEntry->Value.Buffer = NULL;
//
//         /* Copy and initialize the value name */
//         RegEntry->Name.Buffer = (PWCHAR)(RegEntry + 1);
//         RegEntry->Name.Length = NameString.Length;
//         RegEntry->Name.MaximumLength = NameString.MaximumLength;
//         RtlCopyMemory(RegEntry->Name.Buffer,
//                       NameString.Buffer,
//                       NameString.MaximumLength);
//
//         /* Add this entry into the list */
//         InsertTailList(ListAddress, &RegEntry->Entry);
//     }
//
//     /* Did we have an old value buffer? */
//     if (RegEntry->Value.Buffer)
//     {
//         /* Free it */
//         ASSERT(RegEntry->Value.Length != 0);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->Value.Buffer);
//     }
//
//     /* Is there no value associated? */
//     if (!Value)
//     {
//         /* We're done here */
//         RtlInitUnicodeString(&RegEntry->Value, NULL);
//         return STATUS_SUCCESS;
//     }
//
//     /* There is a value, so allocate a buffer for it */
//     RegEntry->Value.Buffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                              SmBaseTag,
//                                              ValueString.MaximumLength);
//     if (!RegEntry->Value.Buffer)
//     {
//         /* Out of memory, undo */
//         RemoveEntryList(&RegEntry->Entry);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Copy the value into the entry */
//     RegEntry->Value.Length = ValueString.Length;
//     RegEntry->Value.MaximumLength = ValueString.MaximumLength;
//     RtlCopyMemory(RegEntry->Value.Buffer,
//                   ValueString.Buffer,
//                   ValueString.MaximumLength);
//
//     /* Now allocate memory for an ANSI copy of it */
//     RegEntry->AnsiValue = RtlAllocateHeap(RtlGetProcessHeap(),
//                                           SmBaseTag,
//                                           (ValueString.Length / sizeof(WCHAR)) +
//                                           sizeof(ANSI_NULL));
//     if (!RegEntry->AnsiValue)
//     {
//         /* Out of memory, undo */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->Value.Buffer);
//         RemoveEntryList(&RegEntry->Entry);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Convert the Unicode value string and return success */
//     RtlInitEmptyAnsiString(&AnsiValueString,
//                            RegEntry->AnsiValue,
//                            (ValueString.Length / sizeof(WCHAR)) +
//                            sizeof(ANSI_NULL));
//     RtlUnicodeStringToAnsiString(&AnsiValueString, &ValueString, FALSE);
//     return STATUS_SUCCESS;
// }
//
// PSMP_REGISTRY_VALUE
// NTAPI
// SmpFindRegistryValue(IN PLIST_ENTRY List,
//                      IN PWSTR ValueName)
// {
//     PSMP_REGISTRY_VALUE RegEntry;
//     UNICODE_STRING ValueString;
//     PLIST_ENTRY NextEntry;
//
//     /* Initialize the value name sting */
//     RtlInitUnicodeString(&ValueString, ValueName);
//
//     /* Loop the list */
//     NextEntry = List->Flink;
//     while (NextEntry != List)
//     {
//         /* Get each entry */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//
//         /* Check if the value name matches */
//         if (!RtlCompareUnicodeString(&RegEntry->Name, &ValueString, TRUE)) break;
//
//         /* It doesn't, move on */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* If we looped back, return NULL, otherwise return the entry we found */
//     if (NextEntry == List) RegEntry = NULL;
//     return RegEntry;
// }
//
// NTSTATUS
// NTAPI
// SmpConfigureProtectionMode(IN PWSTR ValueName,
//                            IN ULONG ValueType,
//                            IN PVOID ValueData,
//                            IN ULONG ValueLength,
//                            IN PVOID Context,
//                            IN PVOID EntryContext)
// {
//     /* Make sure the value is valid */
//     if (ValueLength == sizeof(ULONG))
//     {
//         /* Read it */
//         SmpProtectionMode = *(PULONG)ValueData;
//     }
//     else
//     {
//         /* Default is to protect stuff */
//         SmpProtectionMode = 1;
//     }
//
//     /* Recreate the security descriptors to take into account security mode */
//     SmpCreateSecurityDescriptors(FALSE);
//     DPRINT("SmpProtectionMode: %lu\n", SmpProtectionMode);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpConfigureAllowProtectedRenames(IN PWSTR ValueName,
//                                   IN ULONG ValueType,
//                                   IN PVOID ValueData,
//                                   IN ULONG ValueLength,
//                                   IN PVOID Context,
//                                   IN PVOID EntryContext)
// {
//     /* Make sure the value is valid */
//     if (ValueLength == sizeof(ULONG))
//     {
//         /* Read it */
//         SmpAllowProtectedRenames = *(PULONG)ValueData;
//     }
//     else
//     {
//         /* Default is to not allow protected renames */
//         SmpAllowProtectedRenames = 0;
//     }
//
//     DPRINT("SmpAllowProtectedRenames: %lu\n", SmpAllowProtectedRenames);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpConfigureObjectDirectories(IN PWSTR ValueName,
//                               IN ULONG ValueType,
//                               IN PVOID ValueData,
//                               IN ULONG ValueLength,
//                               IN PVOID Context,
//                               IN PVOID EntryContext)
// {
//     PISECURITY_DESCRIPTOR SecDescriptor;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE DirHandle;
//     UNICODE_STRING RpcString, WindowsString, SearchString;
//     PWCHAR SourceString = ValueData;
//
//     /* Initialize the two strings we will be looking for */
//     RtlInitUnicodeString(&RpcString, L"\\RPC Control");
//     RtlInitUnicodeString(&WindowsString, L"\\Windows");
//
//     /* Loop the registry data we received */
//     while (*SourceString)
//     {
//         /* Assume primary SD for most objects */
//         RtlInitUnicodeString(&SearchString, SourceString);
//         SecDescriptor = SmpPrimarySecurityDescriptor;
//
//         /* But for these two always set the liberal descriptor */
//         if ((RtlEqualUnicodeString(&SearchString, &RpcString, TRUE)) ||
//             (RtlEqualUnicodeString(&SearchString, &WindowsString, TRUE)))
//         {
//             SecDescriptor = SmpLiberalSecurityDescriptor;
//         }
//
//         /* Create the requested directory with the requested descriptor */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &SearchString,
//                                    OBJ_CASE_INSENSITIVE |
//                                    OBJ_OPENIF |
//                                    OBJ_PERMANENT,
//                                    NULL,
//                                    SecDescriptor);
//         DPRINT("Creating: %wZ directory\n", &SearchString);
//         Status = NtCreateDirectoryObject(&DirHandle,
//                                          DIRECTORY_ALL_ACCESS,
//                                          &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Failure case */
//             DPRINT1("SMSS: Unable to create %wZ object directory - Status == %lx\n",
//                     &SearchString, Status);
//         }
//         else
//         {
//             /* It worked, now close the handle */
//             NtClose(DirHandle);
//         }
//
//         /* Move to the next requested object */
//         SourceString += wcslen(SourceString) + 1;
//     }
//
//     /* All done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpConfigureMemoryMgmt(IN PWSTR ValueName,
//                        IN ULONG ValueType,
//                        IN PVOID ValueData,
//                        IN ULONG ValueLength,
//                        IN PVOID Context,
//                        IN PVOID EntryContext)
// {
//     /* Save this is into a list */
//     return SmpSaveRegistryValue(EntryContext, ValueData, NULL, TRUE);
// }
//
// NTSTATUS
// NTAPI
// SmpConfigureFileRenames(IN PWSTR ValueName,
//                         IN ULONG ValueType,
//                         IN PVOID ValueData,
//                         IN ULONG ValueLength,
//                         IN PVOID Context,
//                         IN PVOID EntryContext)
// {
//     NTSTATUS Status;
//     static PWCHAR Canary = NULL;
//
//     /* Check if this is the second call */
//     if (Canary)
//     {
//         /* Save the data into the list */
//         DPRINT("Renamed file: '%S' - '%S'\n", Canary, ValueData);
//         Status = SmpSaveRegistryValue(EntryContext, Canary, ValueData, FALSE);
//         Canary = NULL;
//     }
//     else
//     {
//         /* This it the first call, do nothing until we get the second call */
//         Canary = ValueData;
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpConfigureExcludeKnownDlls(IN PWSTR ValueName,
//                              IN ULONG ValueType,
//                              IN PVOID ValueData,
//                              IN ULONG ValueLength,
//                              IN PVOID Context,
//                              IN PVOID EntryContext)
// {
//     PWCHAR DllName;
//     NTSTATUS Status;
//
//     /* Make sure the value type is valid */
//     if ((ValueType == REG_MULTI_SZ) || (ValueType == REG_SZ))
//     {
//         /* Keep going for each DLL in the list */
//         DllName = ValueData;
//         while (*DllName)
//         {
//             /* Add this to the linked list */
//             DPRINT("Excluded DLL: %S\n", DllName);
//             Status = SmpSaveRegistryValue(EntryContext, DllName, NULL, TRUE);
//
//             /* Bail out on failure or if only one DLL name was present */
//             if (!(NT_SUCCESS(Status)) || (ValueType == REG_SZ)) return Status;
//
//             /* Otherwise, move to the next DLL name */
//             DllName += wcslen(DllName) + 1;
//         }
//     }
//
//     /* All done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpConfigureDosDevices(IN PWSTR ValueName,
//                        IN ULONG ValueType,
//                        IN PVOID ValueData,
//                        IN ULONG ValueLength,
//                        IN PVOID Context,
//                        IN PVOID EntryContext)
// {
//     /* Save into linked list */
//     return SmpSaveRegistryValue(EntryContext, ValueName, ValueData, TRUE);
// }
//
// NTSTATUS
// NTAPI
// SmpInitializeKnownDllPath(IN PUNICODE_STRING DllPath,
//                           IN PWCHAR Buffer,
//                           IN ULONG Length)
// {
//     NTSTATUS Status;
//
//     /* Allocate the buffer */
//     DllPath->Buffer = RtlAllocateHeap(RtlGetProcessHeap(), SmBaseTag, Length);
//     if (DllPath->Buffer)
//     {
//         /* Fill out the rest of the string */
//         DllPath->MaximumLength = (USHORT)Length;
//         DllPath->Length = (USHORT)Length - sizeof(UNICODE_NULL);
//
//         /* Copy the actual path and return success */
//         RtlCopyMemory(DllPath->Buffer, Buffer, Length);
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Fail with out of memory code */
//         Status = STATUS_NO_MEMORY;
//     }
//
//     /* Return result */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpConfigureKnownDlls(IN PWSTR ValueName,
//                       IN ULONG ValueType,
//                       IN PVOID ValueData,
//                       IN ULONG ValueLength,
//                       IN PVOID Context,
//                       IN PVOID EntryContext)
// {
//     /* Check which value is being set */
//     if (_wcsicmp(ValueName, L"DllDirectory") == 0)
//     {
//         /* This is the directory, initialize it */
//         DPRINT("KnownDll Path: %S\n", ValueData);
//         return SmpInitializeKnownDllPath(&SmpKnownDllPath, ValueData, ValueLength);
//     }
//     else
//     {
//         /* Add to the linked list -- this is a file */
//         return SmpSaveRegistryValue(EntryContext, ValueName, ValueData, TRUE);
//     }
// }
//
// /**
//  * @remark
//  * SmpConfigureEnvironment() should be called twice in order to resolve
//  * forward references to environment variables.
//  * See the two L"Environment" entries in SmpRegistryConfigurationTable[].
//  **/
// NTSTATUS
// NTAPI
// SmpConfigureEnvironment(IN PWSTR ValueName,
//                         IN ULONG ValueType,
//                         IN PVOID ValueData,
//                         IN ULONG ValueLength,
//                         IN PVOID Context,
//                         IN PVOID EntryContext)
// {
//     NTSTATUS Status;
//     UNICODE_STRING ValueString, DataString;
//
//     /* Convert the strings into UNICODE_STRING and set the variable defined */
//     RtlInitUnicodeString(&ValueString, ValueName);
//     RtlInitUnicodeString(&DataString, ValueData);
//     DPRINT("Setting %wZ = %wZ\n", &ValueString, &DataString);
//     Status = RtlSetEnvironmentVariable(NULL, &ValueString, &DataString);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: 'SET %wZ = %wZ' failed - Status == %lx\n",
//                 &ValueString, &DataString, Status);
//         return Status;
//     }
//
//     /* Check if the path is being set, and wait for the second instantiation */
//     if ((_wcsicmp(ValueName, L"Path") == 0) && (++SmpCalledConfigEnv == 2))
//     {
//         /* Allocate the path buffer */
//         SmpDefaultLibPathBuffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                   SmBaseTag,
//                                                   ValueLength);
//         if (!SmpDefaultLibPathBuffer) return STATUS_NO_MEMORY;
//
//         /* Copy the data into it and create the UNICODE_STRING to hold it */
//         RtlCopyMemory(SmpDefaultLibPathBuffer, ValueData, ValueLength);
//         RtlInitUnicodeString(&SmpDefaultLibPath, SmpDefaultLibPathBuffer);
//     }
//
//     /* All good */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpConfigureSubSystems(IN PWSTR ValueName,
//                        IN ULONG ValueType,
//                        IN PVOID ValueData,
//                        IN ULONG ValueLength,
//                        IN PVOID Context,
//                        IN PVOID EntryContext)
// {
//     PSMP_REGISTRY_VALUE RegEntry;
//     PWCHAR SubsystemName;
//
//     /* Debug: log what we receive from registry */
//     DPRINT1("SmpConfigureSubSystems: ValueName='%S' ValueType=%lu ValueLength=%lu ValueData='%S'\n",
//             ValueName, ValueType, ValueLength,
//             (ValueType == REG_SZ || ValueType == REG_EXPAND_SZ) ? (PWCHAR)ValueData : L"(non-string)");
//
//     /* Is this a required or optional subsystem? */
//     if ((_wcsicmp(ValueName, L"Required") != 0) &&
//         (_wcsicmp(ValueName, L"Optional") != 0))
//     {
//         /* It isn't, is this the PSI flag? */
//         if ((_wcsicmp(ValueName, L"PosixSingleInstance") != 0) ||
//             (ValueType != REG_DWORD))
//         {
//             /* It isn't, must be a subsystem entry, add it to the list */
//             DPRINT("Subsystem entry: %S-%S\n", ValueName, ValueData);
//             return SmpSaveRegistryValue(EntryContext, ValueName, ValueData, TRUE);
//         }
//
//         /* This was the PSI flag, save it and exit */
//         RegPosixSingleInstance = TRUE;
//         return STATUS_SUCCESS;
//     }
//
//     /* This should be one of the required/optional lists. Is the type valid? */
//     if (ValueType == REG_MULTI_SZ)
//     {
//         /* It is, get the first subsystem */
//         SubsystemName = ValueData;
//         while (*SubsystemName)
//         {
//             /* We should have already put it into the list when we found it */
//             DPRINT("Found subsystem: %S\n", SubsystemName);
//             RegEntry = SmpFindRegistryValue(EntryContext, SubsystemName);
//             if (!RegEntry)
//             {
//                 /* This subsystem doesn't exist, so skip it */
//                 DPRINT1("SMSS: Invalid subsystem name - %ws\n", SubsystemName);
//             }
//             else
//             {
//                 /* Found it -- remove it from the main list */
//                 RemoveEntryList(&RegEntry->Entry);
//
//                 /* Figure out which list to put it in */
//                 if (_wcsicmp(ValueName, L"Required") == 0)
//                 {
//                     /* Put it into the required list */
//                     DPRINT("Required\n");
//                     InsertTailList(&SmpSubSystemsToLoad, &RegEntry->Entry);
//                 }
//                 else
//                 {
//                     /* Put it into the optional list */
//                     DPRINT("Optional\n");
//                     InsertTailList(&SmpSubSystemsToDefer, &RegEntry->Entry);
//                 }
//             }
//
//             /* Move to the next name */
//             SubsystemName += wcslen(SubsystemName) + 1;
//         }
//     }
//
//     /* All done! */
//     return STATUS_SUCCESS;
// }
//
// RTL_QUERY_REGISTRY_TABLE
// SmpRegistryConfigurationTable[] =
// {
//     {
//         SmpConfigureProtectionMode,
//         0,
//         L"ProtectionMode",
//         NULL,
//         REG_DWORD,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureAllowProtectedRenames,
//         RTL_QUERY_REGISTRY_DELETE,
//         L"AllowProtectedRenames",
//         NULL,
//         REG_DWORD,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureObjectDirectories,
//         0,
//         L"ObjectDirectories",
//         NULL,
//         REG_MULTI_SZ,
//         L"\\Windows\0\\RPC Control\0",
//         0
//     },
//
//     {
//         SmpConfigureMemoryMgmt,
//         0,
//         L"BootExecute",
//         &SmpBootExecuteList,
//         REG_MULTI_SZ,
//         L"autocheck AutoChk.exe *\0",
//         0
//     },
//
//     {
//         SmpConfigureMemoryMgmt,
//         RTL_QUERY_REGISTRY_TOPKEY,
//         L"SetupExecute",
//         &SmpSetupExecuteList,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureFileRenames,
//         RTL_QUERY_REGISTRY_DELETE,
//         L"PendingFileRenameOperations",
//         &SmpFileRenameList,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureFileRenames,
//         RTL_QUERY_REGISTRY_DELETE,
//         L"PendingFileRenameOperations2",
//         &SmpFileRenameList,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureExcludeKnownDlls,
//         0,
//         L"ExcludeFromKnownDlls",
//         &SmpExcludeKnownDllsList,
//         REG_MULTI_SZ,
//         L"\0",
//         0
//     },
//
//     {
//         NULL,
//         RTL_QUERY_REGISTRY_SUBKEY,
//         L"Memory Management",
//         NULL,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureMemoryMgmt,
//         0,
//         L"PagingFiles",
//         &SmpPagingFileList,
//         REG_MULTI_SZ,
//         L"?:\\pagefile.sys\0",
//         0
//     },
//
//     {
//         SmpConfigureDosDevices,
//         RTL_QUERY_REGISTRY_SUBKEY,
//         L"DOS Devices",
//         &SmpDosDevicesList,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureKnownDlls,
//         RTL_QUERY_REGISTRY_SUBKEY,
//         L"KnownDlls",
//         &SmpKnownDllsList,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     /**
//      * @remark
//      * SmpConfigureEnvironment() is expected to be called twice
//      * (see SmpCalledConfigEnv) in order to resolve forward references
//      * to environment variables (e.g. EnvVar1 referring to EnvVar2,
//      * before EnvVar2 is defined).
//      **/
//     {
//         SmpConfigureEnvironment,
//         RTL_QUERY_REGISTRY_SUBKEY,
//         L"Environment",
//         NULL,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureEnvironment,
//         RTL_QUERY_REGISTRY_SUBKEY,
//         L"Environment",
//         NULL,
//         REG_NONE,
//         NULL,
//         0
//     },
//     /****/
//
//     {
//         SmpConfigureSubSystems,
//         RTL_QUERY_REGISTRY_SUBKEY,
//         L"SubSystems",
//         &SmpSubSystemList,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureSubSystems,
//         RTL_QUERY_REGISTRY_NOEXPAND,
//         L"Required",
//         &SmpSubSystemList,
//         REG_MULTI_SZ,
//         L"Debug\0Windows\0",
//         0
//     },
//
//     {
//         SmpConfigureSubSystems,
//         RTL_QUERY_REGISTRY_NOEXPAND,
//         L"Optional",
//         &SmpSubSystemList,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureSubSystems,
//         0,
//         L"Kmode",
//         &SmpSubSystemList,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {
//         SmpConfigureMemoryMgmt,
//         RTL_QUERY_REGISTRY_TOPKEY,
//         L"Execute",
//         &SmpExecuteList,
//         REG_NONE,
//         NULL,
//         0
//     },
//
//     {0},
// };
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// NTAPI
// SmpTranslateSystemPartitionInformation(VOID)
// {
//     NTSTATUS Status;
//     UNICODE_STRING UnicodeString, LinkTarget, SearchString, SystemPartition;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle, LinkHandle;
//     ULONG Length, Context;
//     size_t StrLength;
//     WCHAR LinkBuffer[MAX_PATH];
//     CHAR ValueBuffer[sizeof(KEY_VALUE_PARTIAL_INFORMATION) + 512];
//     PKEY_VALUE_PARTIAL_INFORMATION PartialInfo = (PVOID)ValueBuffer;
//     CHAR DirInfoBuffer[sizeof(OBJECT_DIRECTORY_INFORMATION) + 512];
//     POBJECT_DIRECTORY_INFORMATION DirInfo = (PVOID)DirInfoBuffer;
//
//     /* Open the setup key */
//     RtlInitUnicodeString(&UnicodeString, L"\\Registry\\Machine\\System\\Setup");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &UnicodeString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle, KEY_READ, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Cannot open system setup key for reading: 0x%x\n", Status);
//         return;
//     }
//
//     /* Query the system partition */
//     RtlInitUnicodeString(&UnicodeString, L"SystemPartition");
//     Status = NtQueryValueKey(KeyHandle,
//                              &UnicodeString,
//                              KeyValuePartialInformation,
//                              PartialInfo,
//                              sizeof(ValueBuffer),
//                              &Length);
//     NtClose(KeyHandle);
//     if (!NT_SUCCESS(Status) ||
//         ((PartialInfo->Type != REG_SZ) && (PartialInfo->Type != REG_EXPAND_SZ)))
//     {
//         DPRINT1("SMSS: Cannot query SystemPartition value (Type %lu, Status 0x%x)\n",
//                 PartialInfo->Type, Status);
//         return;
//     }
//
//     /* Initialize the system partition string */
//     RtlInitEmptyUnicodeString(&SystemPartition,
//                               (PWCHAR)PartialInfo->Data,
//                               PartialInfo->DataLength);
//     RtlStringCbLengthW(SystemPartition.Buffer,
//                        SystemPartition.MaximumLength,
//                        &StrLength);
//     SystemPartition.Length = (USHORT)StrLength;
//
//     /* Enumerate the directory looking for the symbolic link string */
//     RtlInitUnicodeString(&SearchString, L"SymbolicLink");
//     RtlInitEmptyUnicodeString(&LinkTarget, LinkBuffer, sizeof(LinkBuffer));
//     Status = NtQueryDirectoryObject(SmpDosDevicesObjectDirectory,
//                                     DirInfo,
//                                     sizeof(DirInfoBuffer),
//                                     TRUE,
//                                     TRUE,
//                                     &Context,
//                                     NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Cannot find drive letter for system partition\n");
//         return;
//     }
//
//     /* Keep searching until we find it */
//     do
//     {
//         /* Is this it? */
//         if ((RtlEqualUnicodeString(&DirInfo->TypeName, &SearchString, TRUE)) &&
//             (DirInfo->Name.Length == 2 * sizeof(WCHAR)) &&
//             (DirInfo->Name.Buffer[1] == L':'))
//         {
//             /* Looks like we found it, open the link to get its target */
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &DirInfo->Name,
//                                        OBJ_CASE_INSENSITIVE,
//                                        SmpDosDevicesObjectDirectory,
//                                        NULL);
//             Status = NtOpenSymbolicLinkObject(&LinkHandle,
//                                               SYMBOLIC_LINK_ALL_ACCESS,
//                                               &ObjectAttributes);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Open worked, query the target now */
//                 Status = NtQuerySymbolicLinkObject(LinkHandle,
//                                                    &LinkTarget,
//                                                    NULL);
//                 NtClose(LinkHandle);
//
//                 /* Check if it matches the string we had found earlier */
//                 if ((NT_SUCCESS(Status)) &&
//                     ((RtlEqualUnicodeString(&SystemPartition,
//                                             &LinkTarget,
//                                             TRUE)) ||
//                     ((RtlPrefixUnicodeString(&SystemPartition,
//                                              &LinkTarget,
//                                              TRUE)) &&
//                      (LinkTarget.Buffer[SystemPartition.Length / sizeof(WCHAR)] == L'\\'))))
//                 {
//                     /* All done */
//                     break;
//                 }
//             }
//         }
//
//         /* Couldn't find it, try again */
//         Status = NtQueryDirectoryObject(SmpDosDevicesObjectDirectory,
//                                         DirInfo,
//                                         sizeof(DirInfoBuffer),
//                                         TRUE,
//                                         FALSE,
//                                         &Context,
//                                         NULL);
//     } while (NT_SUCCESS(Status));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Cannot find drive letter for system partition\n");
//         return;
//     }
//
//     /* Open the setup key again, for full access this time */
//     RtlInitUnicodeString(&UnicodeString,
//                          L"\\Registry\\Machine\\Software\\Microsoft\\Windows\\CurrentVersion\\Setup");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &UnicodeString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle, KEY_ALL_ACCESS, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Cannot open software setup key for writing: 0x%x\n",
//                 Status);
//         return;
//     }
//
//     /* Wrap up the end of the link buffer */
//     wcsncpy(LinkBuffer, DirInfo->Name.Buffer, 2);
//     LinkBuffer[2] = L'\\';
//     LinkBuffer[3] = L'\0';
//
//     /* Now set this as the "BootDir" */
//     RtlInitUnicodeString(&UnicodeString, L"BootDir");
//     Status = NtSetValueKey(KeyHandle,
//                            &UnicodeString,
//                            0,
//                            REG_SZ,
//                            LinkBuffer,
//                            4 * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: couldn't write BootDir value: 0x%x\n", Status);
//     }
//     NtClose(KeyHandle);
// }
//
// NTSTATUS
// NTAPI
// SmpCreateSecurityDescriptors(IN BOOLEAN InitialCall)
// {
//     NTSTATUS Status;
//     PSID WorldSid = NULL, AdminSid = NULL, SystemSid = NULL;
//     PSID RestrictedSid = NULL, OwnerSid = NULL;
//     SID_IDENTIFIER_AUTHORITY WorldAuthority = {SECURITY_WORLD_SID_AUTHORITY};
//     SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//     SID_IDENTIFIER_AUTHORITY CreatorAuthority = {SECURITY_CREATOR_SID_AUTHORITY};
//     ULONG AclLength, SidLength;
//     PACL Acl;
//     PACE_HEADER Ace;
//     BOOLEAN ProtectionRequired = FALSE;
//
//     /* Check if this is the first call */
//     if (InitialCall)
//     {
//         /* Create and set the primary descriptor */
//         SmpPrimarySecurityDescriptor = &SmpPrimarySDBody;
//         Status = RtlCreateSecurityDescriptor(SmpPrimarySecurityDescriptor,
//                                              SECURITY_DESCRIPTOR_REVISION);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlSetDaclSecurityDescriptor(SmpPrimarySecurityDescriptor,
//                                               TRUE,
//                                               NULL,
//                                               FALSE);
//         ASSERT(NT_SUCCESS(Status));
//
//         /* Create and set the liberal descriptor */
//         SmpLiberalSecurityDescriptor = &SmpLiberalSDBody;
//         Status = RtlCreateSecurityDescriptor(SmpLiberalSecurityDescriptor,
//                                              SECURITY_DESCRIPTOR_REVISION);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlSetDaclSecurityDescriptor(SmpLiberalSecurityDescriptor,
//                                               TRUE,
//                                               NULL,
//                                               FALSE);
//         ASSERT(NT_SUCCESS(Status));
//
//         /* Create and set the \KnownDlls descriptor */
//         SmpKnownDllsSecurityDescriptor = &SmpKnownDllsSDBody;
//         Status = RtlCreateSecurityDescriptor(SmpKnownDllsSecurityDescriptor,
//                                              SECURITY_DESCRIPTOR_REVISION);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlSetDaclSecurityDescriptor(SmpKnownDllsSecurityDescriptor,
//                                               TRUE,
//                                               NULL,
//                                               FALSE);
//         ASSERT(NT_SUCCESS(Status));
//
//         /* Create and Set the \ApiPort descriptor */
//         SmpApiPortSecurityDescriptor = &SmpApiPortSDBody;
//         Status = RtlCreateSecurityDescriptor(SmpApiPortSecurityDescriptor,
//                                              SECURITY_DESCRIPTOR_REVISION);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlSetDaclSecurityDescriptor(SmpApiPortSecurityDescriptor,
//                                               TRUE,
//                                               NULL,
//                                               FALSE);
//         ASSERT(NT_SUCCESS(Status));
//     }
//
//     /* Check if protection was requested in the registry (on by default) */
//     if (SmpProtectionMode & 1) ProtectionRequired = TRUE;
//
//     /* Exit if there's nothing to do */
//     if (!(InitialCall || ProtectionRequired)) return STATUS_SUCCESS;
//
//     /* Build the world SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority, 1,
//                                          SECURITY_WORLD_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &WorldSid);
//     if (!NT_SUCCESS(Status))
//     {
//         WorldSid = NULL;
//         goto Quickie;
//     }
//
//     /* Build the admin SID */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority, 2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ADMINS,
//                                          0, 0, 0, 0, 0, 0,
//                                          &AdminSid);
//     if (!NT_SUCCESS(Status))
//     {
//         AdminSid = NULL;
//         goto Quickie;
//     }
//
//     /* Build the owner SID */
//     Status = RtlAllocateAndInitializeSid(&CreatorAuthority, 1,
//                                          SECURITY_CREATOR_OWNER_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &OwnerSid);
//     if (!NT_SUCCESS(Status))
//     {
//         OwnerSid = NULL;
//         goto Quickie;
//     }
//
//     /* Build the restricted SID */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority, 1,
//                                          SECURITY_RESTRICTED_CODE_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &RestrictedSid);
//     if (!NT_SUCCESS(Status))
//     {
//         RestrictedSid = NULL;
//         goto Quickie;
//     }
//
//     /* Build the system SID */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority, 1,
//                                          SECURITY_LOCAL_SYSTEM_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &SystemSid);
//     if (!NT_SUCCESS(Status))
//     {
//         SystemSid = NULL;
//         goto Quickie;
//     }
//
//     /* Now check if we're creating the core descriptors */
//     if (!InitialCall)
//     {
//         /* We're skipping NextAcl so we have to do this here */
//         SidLength = RtlLengthSid(WorldSid) + RtlLengthSid(RestrictedSid) + RtlLengthSid(AdminSid);
//         SidLength *= 2;
//         goto NotInitial;
//     }
//
//     /* Allocate an ACL with two ACEs with two SIDs each */
//     SidLength = RtlLengthSid(SystemSid) + RtlLengthSid(AdminSid);
//     AclLength = sizeof(ACL) + 2 * sizeof(ACCESS_ALLOWED_ACE) + SidLength;
//     Acl = RtlAllocateHeap(RtlGetProcessHeap(), 0, AclLength);
//     if (!Acl) Status = STATUS_NO_MEMORY;
//     if (!NT_SUCCESS(Status)) goto NextAcl;
//
//     /* Now build the ACL and add the two ACEs */
//     Status = RtlCreateAcl(Acl, AclLength, ACL_REVISION2);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, AdminSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, SystemSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Set this as the DACL */
//     Status = RtlSetDaclSecurityDescriptor(SmpApiPortSecurityDescriptor,
//                                           TRUE,
//                                           Acl,
//                                           FALSE);
//     ASSERT(NT_SUCCESS(Status));
//
// NextAcl:
//     /* Allocate an ACL with 6 ACEs, two ACEs per SID */
//     SidLength = RtlLengthSid(WorldSid) + RtlLengthSid(RestrictedSid) + RtlLengthSid(AdminSid);
//     SidLength *= 2;
//     AclLength = sizeof(ACL) + 6 * sizeof(ACCESS_ALLOWED_ACE) + SidLength;
//     Acl = RtlAllocateHeap(RtlGetProcessHeap(), 0, AclLength);
//     if (!Acl) Status = STATUS_NO_MEMORY;
//     if (!NT_SUCCESS(Status)) goto NotInitial;
//
//     /* Now build the ACL and add the six ACEs */
//     Status = RtlCreateAcl(Acl, AclLength, ACL_REVISION2);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE, WorldSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE, RestrictedSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, AdminSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE, WorldSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE, RestrictedSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, AdminSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Now edit the last three ACEs and make them inheritable */
//     Status = RtlGetAce(Acl, 3, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//     Status = RtlGetAce(Acl, 4, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//     Status = RtlGetAce(Acl, 5, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//
//     /* Set this as the DACL */
//     Status = RtlSetDaclSecurityDescriptor(SmpKnownDllsSecurityDescriptor,
//                                           TRUE,
//                                           Acl,
//                                           FALSE);
//     ASSERT(NT_SUCCESS(Status));
//
// NotInitial:
//     /* The initial ACLs have been created, are we also protecting objects? */
//     if (!ProtectionRequired) goto Quickie;
//
//     /* Allocate an ACL with 7 ACEs, two ACEs per SID, and one final owner ACE */
//     SidLength += RtlLengthSid(OwnerSid);
//     AclLength = sizeof(ACL) + 7 * sizeof (ACCESS_ALLOWED_ACE) + 2 * SidLength;
//     Acl = RtlAllocateHeap(RtlGetProcessHeap(), 0, AclLength);
//     if (!Acl) Status = STATUS_NO_MEMORY;
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Build the ACL and add the seven ACEs */
//     Status = RtlCreateAcl(Acl, AclLength, ACL_REVISION2);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ, WorldSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ, RestrictedSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, AdminSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ, WorldSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ, RestrictedSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, AdminSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, OwnerSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Edit the last 4 ACEs to make then inheritable */
//     Status = RtlGetAce(Acl, 3, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//     Status = RtlGetAce(Acl, 4, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//     Status = RtlGetAce(Acl, 5, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//     Status = RtlGetAce(Acl, 6, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//
//     /* Set this as the DACL for the primary SD */
//     Status = RtlSetDaclSecurityDescriptor(SmpPrimarySecurityDescriptor,
//                                           TRUE,
//                                           Acl,
//                                           FALSE);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Allocate an ACL with 7 ACEs, two ACEs per SID, and one final owner ACE */
//     AclLength = sizeof(ACL) + 7 * sizeof (ACCESS_ALLOWED_ACE) + 2 * SidLength;
//     Acl = RtlAllocateHeap(RtlGetProcessHeap(), 0, AclLength);
//     if (!Acl) Status = STATUS_NO_MEMORY;
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Build the ACL and add the seven ACEs */
//     Status = RtlCreateAcl(Acl, AclLength, ACL_REVISION2);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE, WorldSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE, RestrictedSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, AdminSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE, WorldSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_EXECUTE | GENERIC_READ | GENERIC_WRITE, RestrictedSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, AdminSid);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlAddAccessAllowedAce(Acl, ACL_REVISION2, GENERIC_ALL, OwnerSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Edit the last 4 ACEs to make then inheritable */
//     Status = RtlGetAce(Acl, 3, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//     Status = RtlGetAce(Acl, 4, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//     Status = RtlGetAce(Acl, 5, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//     Status = RtlGetAce(Acl, 6, (PVOID)&Ace);
//     ASSERT(NT_SUCCESS(Status));
//     Ace->AceFlags = OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE;
//
//     /* Now set this as the DACL for the liberal SD */
//     Status = RtlSetDaclSecurityDescriptor(SmpLiberalSecurityDescriptor,
//                                           TRUE,
//                                           Acl,
//                                           FALSE);
//     ASSERT(NT_SUCCESS(Status));
//
// Quickie:
//     /* Cleanup the SIDs */
//     if (OwnerSid) RtlFreeHeap(RtlGetProcessHeap(), 0, OwnerSid);
//     if (AdminSid) RtlFreeHeap(RtlGetProcessHeap(), 0, AdminSid);
//     if (WorldSid) RtlFreeHeap(RtlGetProcessHeap(), 0, WorldSid);
//     if (SystemSid) RtlFreeHeap(RtlGetProcessHeap(), 0, SystemSid);
//     if (RestrictedSid) RtlFreeHeap(RtlGetProcessHeap(), 0, RestrictedSid);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpInitializeDosDevices(VOID)
// {
//     NTSTATUS Status;
//     PSMP_REGISTRY_VALUE RegEntry;
//     SECURITY_DESCRIPTOR_CONTROL OldFlag = 0;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING GlobalName;
//     HANDLE DirHandle;
//     PLIST_ENTRY NextEntry, Head;
//
//     /* Open the \GLOBAL?? directory */
//     RtlInitUnicodeString(&GlobalName, L"\\??");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &GlobalName,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//     Status = NtOpenDirectoryObject(&SmpDosDevicesObjectDirectory,
//                                    DIRECTORY_ALL_ACCESS,
//                                    &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Unable to open %wZ directory - Status == %lx\n",
//                 &GlobalName, Status);
//         return Status;
//     }
//
//     /* Loop the DOS devices */
//     Head = &SmpDosDevicesList;
//     while (!IsListEmpty(Head))
//     {
//         /* Get the entry and remove it */
//         NextEntry = RemoveHeadList(Head);
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//
//         /* Initialize the attributes, and see which descriptor is being used */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &RegEntry->Name,
//                                    OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT,
//                                    SmpDosDevicesObjectDirectory,
//                                    SmpPrimarySecurityDescriptor);
//         if (SmpPrimarySecurityDescriptor)
//         {
//             /* Save the old flag and set it while we create this link */
//             OldFlag = SmpPrimarySecurityDescriptor->Control;
//             SmpPrimarySecurityDescriptor->Control |= SE_DACL_DEFAULTED;
//         }
//
//         /* Create the symbolic link */
//         DPRINT("Creating symlink for %wZ to %wZ\n", &RegEntry->Name, &RegEntry->Value);
//         Status = NtCreateSymbolicLinkObject(&DirHandle,
//                                             SYMBOLIC_LINK_ALL_ACCESS,
//                                             &ObjectAttributes,
//                                             &RegEntry->Value);
//         if (Status == STATUS_OBJECT_NAME_EXISTS)
//         {
//             /* Make it temporary and get rid of the handle */
//             NtMakeTemporaryObject(DirHandle);
//             NtClose(DirHandle);
//
//             /* Treat this as success, and see if we got a name back */
//             Status = STATUS_SUCCESS;
//             if (RegEntry->Value.Length)
//             {
//                 /* Create it now with this name */
//                 ObjectAttributes.Attributes &= ~OBJ_OPENIF;
//                 Status = NtCreateSymbolicLinkObject(&DirHandle,
//                                                     SYMBOLIC_LINK_ALL_ACCESS,
//                                                     &ObjectAttributes,
//                                                     &RegEntry->Value);
//             }
//         }
//
//         /* If we were using a security descriptor, restore the non-defaulted flag */
//         if (ObjectAttributes.SecurityDescriptor)
//         {
//             SmpPrimarySecurityDescriptor->Control = OldFlag;
//         }
//
//         /* Print a failure if we failed to create the symbolic link */
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("SMSS: Unable to create %wZ => %wZ symbolic link object - Status == 0x%lx\n",
//                     &RegEntry->Name,
//                     &RegEntry->Value,
//                     Status);
//             break;
//         }
//
//         /* Close the handle */
//         NtClose(DirHandle);
//
//         /* Free this entry */
//         if (RegEntry->AnsiValue) RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->AnsiValue);
//         if (RegEntry->Value.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->Value.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry);
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// VOID
// NTAPI
// SmpProcessModuleImports(IN PVOID Unused,
//                         IN PCHAR ImportName)
// {
//     ULONG Length = 0;
//     WCHAR Buffer[MAX_PATH];
//     PWCHAR DllName, DllValue;
//     ANSI_STRING ImportString;
//     UNICODE_STRING ImportUnicodeString;
//     NTSTATUS Status;
//
//     /* Skip NTDLL since it's already always mapped */
//     if (!_stricmp(ImportName, "ntdll.dll")) return;
//
//     /* Initialize our strings */
//     RtlInitAnsiString(&ImportString, ImportName);
//     RtlInitEmptyUnicodeString(&ImportUnicodeString, Buffer, sizeof(Buffer));
//     Status = RtlAnsiStringToUnicodeString(&ImportUnicodeString, &ImportString, FALSE);
//     if (!NT_SUCCESS(Status)) return;
//
//     /* Loop to find the DLL file extension */
//     while (Length < ImportUnicodeString.Length)
//     {
//         if (ImportUnicodeString.Buffer[Length / sizeof(WCHAR)] == L'.') break;
//         Length += sizeof(WCHAR);
//     }
//
//     /*
//      * Break up the values as needed; the buffer acquires the form:
//      * "dll_name.dll\0dll_name\0"
//      */
//     DllValue = ImportUnicodeString.Buffer;
//     DllName = &ImportUnicodeString.Buffer[(ImportUnicodeString.Length + sizeof(UNICODE_NULL)) / sizeof(WCHAR)];
//     RtlStringCbCopyNW(DllName,
//                       ImportUnicodeString.MaximumLength - (ImportUnicodeString.Length + sizeof(UNICODE_NULL)),
//                       ImportUnicodeString.Buffer, Length);
//
//     /* Add the DLL to the list */
//     SmpSaveRegistryValue(&SmpKnownDllsList, DllName, DllValue, TRUE);
// }
//
// NTSTATUS
// NTAPI
// SmpInitializeKnownDllsInternal(IN PUNICODE_STRING Directory,
//                                IN PUNICODE_STRING Path)
// {
//     HANDLE DirFileHandle, DirHandle, SectionHandle, FileHandle, LinkHandle;
//     UNICODE_STRING NtPath, SymLinkName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status, Status1;
//     PLIST_ENTRY NextEntry;
//     PSMP_REGISTRY_VALUE RegEntry;
//     ULONG_PTR ErrorParameters[3];
//     UNICODE_STRING ErrorResponse;
//     IO_STATUS_BLOCK IoStatusBlock;
//     SECURITY_DESCRIPTOR_CONTROL OldFlag = 0;
//     USHORT ImageCharacteristics;
//
//     /* Initialize to NULL */
//     DirFileHandle = NULL;
//     DirHandle = NULL;
//     NtPath.Buffer = NULL;
//
//     /* Create the \KnownDLLs directory */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                Directory,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT,
//                                NULL,
//                                SmpKnownDllsSecurityDescriptor);
//     Status = NtCreateDirectoryObject(&DirHandle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Handle failure */
//         DPRINT1("SMSS: Unable to create %wZ directory - Status == %lx\n",
//                 Directory, Status);
//         return Status;
//     }
//
//     /* Convert the path to native format */
//     if (!RtlDosPathNameToNtPathName_U(Path->Buffer, &NtPath, NULL, NULL))
//     {
//         /* Fail if this didn't work */
//         DPRINT1("SMSS: Unable to to convert %wZ to an Nt path\n", Path);
//         Status = STATUS_OBJECT_NAME_INVALID;
//         goto Quickie;
//     }
//
//     /* Open the path that was specified, which should be a directory */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtPath,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenFile(&DirFileHandle,
//                         FILE_LIST_DIRECTORY | SYNCHRONIZE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail if we couldn't open it */
//         DPRINT1("SMSS: Unable to open a handle to the KnownDll directory (%wZ)"
//                 "- Status == %lx\n",
//                 Path,
//                 Status);
//         FileHandle = NULL;
//         goto Quickie;
//     }
//
//     /* Temporarily hack the SD to use a default DACL for this symbolic link */
//     if (SmpPrimarySecurityDescriptor)
//     {
//         OldFlag = SmpPrimarySecurityDescriptor->Control;
//         SmpPrimarySecurityDescriptor->Control |= SE_DACL_DEFAULTED;
//     }
//
//     /* Create a symbolic link to the directory in the object manager */
//     RtlInitUnicodeString(&SymLinkName, L"KnownDllPath");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SymLinkName,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT,
//                                DirHandle,
//                                SmpPrimarySecurityDescriptor);
//     Status = NtCreateSymbolicLinkObject(&LinkHandle,
//                                         SYMBOLIC_LINK_ALL_ACCESS,
//                                         &ObjectAttributes,
//                                         Path);
//
//     /* Undo the hack */
//     if (SmpPrimarySecurityDescriptor) SmpPrimarySecurityDescriptor->Control = OldFlag;
//
//     /* Check if the symlink was created */
//     if (!NT_SUCCESS(Status))
//     {
//         /* It wasn't, so bail out since the OS needs it to exist */
//         DPRINT1("SMSS: Unable to create %wZ symbolic link - Status == %lx\n",
//                 &SymLinkName, Status);
//         LinkHandle = NULL;
//         goto Quickie;
//     }
//
//     /* We created it permanent, we can go ahead and close the handle now */
//     Status1 = NtClose(LinkHandle);
//     ASSERT(NT_SUCCESS(Status1));
//
//     /* Now loop the known DLLs */
//     NextEntry = SmpKnownDllsList.Flink;
//     while (NextEntry != &SmpKnownDllsList)
//     {
//         /* Get the entry and move on */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//         NextEntry = NextEntry->Flink;
//
//         DPRINT("Processing known DLL: %wZ-%wZ\n", &RegEntry->Name, &RegEntry->Value);
//
//         /* Skip the entry if it's in the excluded list */
//         if ((SmpFindRegistryValue(&SmpExcludeKnownDllsList,
//                                   RegEntry->Name.Buffer)) ||
//             (SmpFindRegistryValue(&SmpExcludeKnownDllsList,
//                                   RegEntry->Value.Buffer)))
//         {
//             continue;
//         }
//
//         /* Open the actual file */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &RegEntry->Value,
//                                    OBJ_CASE_INSENSITIVE,
//                                    DirFileHandle,
//                                    NULL);
//         Status1 = NtOpenFile(&FileHandle,
//                              SYNCHRONIZE | FILE_EXECUTE,
//                              &ObjectAttributes,
//                              &IoStatusBlock,
//                              FILE_SHARE_READ | FILE_SHARE_DELETE,
//                              FILE_NON_DIRECTORY_FILE |
//                              FILE_SYNCHRONOUS_IO_NONALERT);
//         /* If we failed, skip it */
//         if (!NT_SUCCESS(Status1)) continue;
//
//         /* Checksum it */
//         Status = LdrVerifyImageMatchesChecksum((HANDLE)((ULONG_PTR)FileHandle | 1),
//                                                SmpProcessModuleImports,
//                                                RegEntry,
//                                                &ImageCharacteristics);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Checksum failed, so don't even try going further -- kill SMSS */
//             RtlInitUnicodeString(&ErrorResponse,
//                                  L"Verification of a KnownDLL failed.");
//             ErrorParameters[0] = (ULONG_PTR)&ErrorResponse;
//             ErrorParameters[1] = Status;
//             ErrorParameters[2] = (ULONG_PTR)&RegEntry->Value;
//             SmpTerminate(ErrorParameters, 5, RTL_NUMBER_OF(ErrorParameters));
//         }
//         else if (!(ImageCharacteristics & IMAGE_FILE_DLL))
//         {
//             /* An invalid known DLL entry will also kill SMSS */
//             RtlInitUnicodeString(&ErrorResponse,
//                                  L"Non-DLL file included in KnownDLL list.");
//             ErrorParameters[0] = (ULONG_PTR)&ErrorResponse;
//             ErrorParameters[1] = STATUS_INVALID_IMPORT_OF_NON_DLL;
//             ErrorParameters[2] = (ULONG_PTR)&RegEntry->Value;
//             SmpTerminate(ErrorParameters, 5, RTL_NUMBER_OF(ErrorParameters));
//         }
//
//         /* Temporarily hack the SD to use a default DACL for this section */
//         if (SmpLiberalSecurityDescriptor)
//         {
//             OldFlag = SmpLiberalSecurityDescriptor->Control;
//             SmpLiberalSecurityDescriptor->Control |= SE_DACL_DEFAULTED;
//         }
//
//         /* Create the section for this known DLL */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &RegEntry->Value,
//                                    OBJ_PERMANENT,
//                                    DirHandle,
//                                    SmpLiberalSecurityDescriptor)
//         Status = NtCreateSection(&SectionHandle,
//                                  SECTION_ALL_ACCESS,
//                                  &ObjectAttributes,
//                                  0,
//                                  PAGE_EXECUTE,
//                                  SEC_IMAGE,
//                                  FileHandle);
//
//         /* Undo the hack */
//         if (SmpLiberalSecurityDescriptor) SmpLiberalSecurityDescriptor->Control = OldFlag;
//
//         /* Check if we created the section okay */
//         if (NT_SUCCESS(Status))
//         {
//             /* We can close it now, since it's marked permanent */
//             Status1 = NtClose(SectionHandle);
//             ASSERT(NT_SUCCESS(Status1));
//         }
//         else
//         {
//             /* If we couldn't make it "known", that's fine and keep going */
//             DPRINT1("SMSS: CreateSection for KnownDll %wZ failed - Status == %lx\n",
//                     &RegEntry->Value, Status);
//         }
//
//         /* Close the file since we can move on to the next one */
//         Status1 = NtClose(FileHandle);
//         ASSERT(NT_SUCCESS(Status1));
//     }
//
// Quickie:
//     /* Close both handles and free the NT path buffer */
//     if (DirHandle)
//     {
//         Status1 = NtClose(DirHandle);
//         ASSERT(NT_SUCCESS(Status1));
//     }
//     if (DirFileHandle)
//     {
//         Status1 = NtClose(DirFileHandle);
//         ASSERT(NT_SUCCESS(Status1));
//     }
//     if (NtPath.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, NtPath.Buffer);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpInitializeKnownDlls(VOID)
// {
//     NTSTATUS Status;
//     PSMP_REGISTRY_VALUE RegEntry;
//     UNICODE_STRING KnownDllsName;
//     PLIST_ENTRY Head, NextEntry;
//
//     /* Call the internal function */
//     RtlInitUnicodeString(&KnownDllsName, L"\\KnownDlls");
//     Status = SmpInitializeKnownDllsInternal(&KnownDllsName, &SmpKnownDllPath);
//
//     /* Wipe out the list regardless of success */
//     Head = &SmpKnownDllsList;
//     while (!IsListEmpty(Head))
//     {
//         /* Remove this entry */
//         NextEntry = RemoveHeadList(Head);
//
//         /* Free it */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->AnsiValue);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->Value.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry);
//     }
//
//     /* All done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpCreateDynamicEnvironmentVariables(VOID)
// {
//     NTSTATUS Status;
//     SYSTEM_BASIC_INFORMATION BasicInfo;
//     SYSTEM_PROCESSOR_INFORMATION ProcessorInfo;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING ValueName, DestinationString;
//     HANDLE KeyHandle, KeyHandle2;
//     PWCHAR ValueData;
//     ULONG ResultLength;
//     size_t StrLength;
//     WCHAR ValueBuffer[sizeof(KEY_VALUE_PARTIAL_INFORMATION) + 512];
//     WCHAR ValueBuffer2[sizeof(KEY_VALUE_PARTIAL_INFORMATION) + 512];
//     PKEY_VALUE_PARTIAL_INFORMATION PartialInfo = (PVOID)ValueBuffer;
//     PKEY_VALUE_PARTIAL_INFORMATION PartialInfo2 = (PVOID)ValueBuffer2;
//
//     /* Get system basic information -- we'll need the CPU count */
//     Status = NtQuerySystemInformation(SystemBasicInformation,
//                                       &BasicInfo,
//                                       sizeof(BasicInfo),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bail out on failure */
//         DPRINT1("SMSS: Unable to query system basic information - %x\n", Status);
//         return Status;
//     }
//
//     /* Get the processor information, we'll query a bunch of revision info */
//     Status = NtQuerySystemInformation(SystemProcessorInformation,
//                                       &ProcessorInfo,
//                                       sizeof(ProcessorInfo),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bail out on failure */
//         DPRINT1("SMSS: Unable to query system processor information - %x\n", Status);
//         return Status;
//     }
//
//     /* We'll be writing all these environment variables over here */
//     RtlInitUnicodeString(&DestinationString,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\"
//                          L"Control\\Session Manager\\Environment");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DestinationString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle, GENERIC_WRITE, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bail out on failure */
//         DPRINT1("SMSS: Unable to open %wZ - %x\n", &DestinationString, Status);
//         return Status;
//     }
//
//     /* First let's write the OS variable */
//     RtlInitUnicodeString(&ValueName, L"OS");
//     ValueData = L"Windows_NT";
//     DPRINT("Setting %wZ to %S\n", &ValueName, ValueData);
//     Status = NtSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_SZ,
//                            ValueData,
//                            (ULONG)(wcslen(ValueData) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Failed writing %wZ environment variable - %x\n",
//                 &ValueName, Status);
//         NtClose(KeyHandle);
//         return Status;
//     }
//
//     /* Next, let's write the CPU architecture variable */
//     RtlInitUnicodeString(&ValueName, L"PROCESSOR_ARCHITECTURE");
//     switch (ProcessorInfo.ProcessorArchitecture)
//     {
//         /* Pick the correct string that matches the architecture */
//         case PROCESSOR_ARCHITECTURE_INTEL:
//             ValueData = L"x86";
//             break;
//
//         case PROCESSOR_ARCHITECTURE_AMD64:
//             ValueData = L"AMD64";
//             break;
//
//         case PROCESSOR_ARCHITECTURE_IA64:
//             ValueData = L"IA64";
//             break;
//
//         default:
//             ValueData = L"Unknown";
//             break;
//     }
//
//     /* Set it */
//     DPRINT("Setting %wZ to %S\n", &ValueName, ValueData);
//     Status = NtSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_SZ,
//                            ValueData,
//                            (ULONG)(wcslen(ValueData) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Failed writing %wZ environment variable - %x\n",
//                 &ValueName, Status);
//         NtClose(KeyHandle);
//         return Status;
//     }
//
//     /* And now let's write the processor level */
//     RtlInitUnicodeString(&ValueName, L"PROCESSOR_LEVEL");
//     swprintf(ValueBuffer, L"%u", ProcessorInfo.ProcessorLevel);
//     DPRINT("Setting %wZ to %S\n", &ValueName, ValueBuffer);
//     Status = NtSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_SZ,
//                            ValueBuffer,
//                            (ULONG)(wcslen(ValueBuffer) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Failed writing %wZ environment variable - %x\n",
//                 &ValueName, Status);
//         NtClose(KeyHandle);
//         return Status;
//     }
//
//     /* Now open the hardware CPU key */
//     RtlInitUnicodeString(&DestinationString,
//                          L"\\Registry\\Machine\\Hardware\\Description\\System\\"
//                          L"CentralProcessor\\0");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DestinationString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle2, KEY_READ, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Unable to open %wZ - %x\n", &DestinationString, Status);
//         NtClose(KeyHandle);
//         return Status;
//     }
//
//     /* So that we can read the identifier out of it... */
//     RtlInitUnicodeString(&ValueName, L"Identifier");
//     Status = NtQueryValueKey(KeyHandle2,
//                              &ValueName,
//                              KeyValuePartialInformation,
//                              PartialInfo,
//                              sizeof(ValueBuffer),
//                              &ResultLength);
//     if (!NT_SUCCESS(Status) ||
//         ((PartialInfo->Type != REG_SZ) && (PartialInfo->Type != REG_EXPAND_SZ)))
//     {
//         NtClose(KeyHandle2);
//         NtClose(KeyHandle);
//         DPRINT1("SMSS: Unable to read %wZ\\%wZ (Type %lu, Status 0x%x)\n",
//                 &DestinationString, &ValueName, PartialInfo->Type, Status);
//         return Status;
//     }
//
//     /* Initialize the string so that it can be large enough
//      * to contain both the identifier and the vendor strings. */
//     RtlInitEmptyUnicodeString(&DestinationString,
//                               (PWCHAR)PartialInfo->Data,
//                               sizeof(ValueBuffer) -
//                                 FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data));
//     RtlStringCbLengthW(DestinationString.Buffer,
//                        PartialInfo->DataLength,
//                        &StrLength);
//     DestinationString.Length = (USHORT)StrLength;
//
//     /* As well as the vendor... */
//     RtlInitUnicodeString(&ValueName, L"VendorIdentifier");
//     Status = NtQueryValueKey(KeyHandle2,
//                              &ValueName,
//                              KeyValuePartialInformation,
//                              PartialInfo2,
//                              sizeof(ValueBuffer2),
//                              &ResultLength);
//     NtClose(KeyHandle2);
//     if (NT_SUCCESS(Status) &&
//         ((PartialInfo2->Type == REG_SZ) || (PartialInfo2->Type == REG_EXPAND_SZ)))
//     {
//         /* To combine it into a single string */
//         RtlStringCbPrintfW(DestinationString.Buffer + DestinationString.Length / sizeof(WCHAR),
//                            DestinationString.MaximumLength - DestinationString.Length,
//                            L", %.*s",
//                            PartialInfo2->DataLength / sizeof(WCHAR),
//                            (PWCHAR)PartialInfo2->Data);
//         DestinationString.Length = (USHORT)(wcslen(DestinationString.Buffer) * sizeof(WCHAR));
//     }
//
//     /* So that we can set this as the PROCESSOR_IDENTIFIER variable */
//     RtlInitUnicodeString(&ValueName, L"PROCESSOR_IDENTIFIER");
//     DPRINT("Setting %wZ to %wZ\n", &ValueName, &DestinationString);
//     Status = NtSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_SZ,
//                            DestinationString.Buffer,
//                            DestinationString.Length + sizeof(UNICODE_NULL));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Failed writing %wZ environment variable - %x\n",
//                 &ValueName, Status);
//         NtClose(KeyHandle);
//         return Status;
//     }
//
//     /* Now let's get the processor architecture */
//     RtlInitUnicodeString(&ValueName, L"PROCESSOR_REVISION");
//     switch (ProcessorInfo.ProcessorArchitecture)
//     {
//         /* Check if this is an older Intel CPU */
//         case PROCESSOR_ARCHITECTURE_INTEL:
//             if ((ProcessorInfo.ProcessorRevision >> 8) == 0xFF)
//             {
//                 /* These guys used a revision + stepping, so get the rev only */
//                 swprintf(ValueBuffer, L"%02x", ProcessorInfo.ProcessorRevision & 0xFF);
//                 _wcsupr(ValueBuffer);
//                 break;
//             }
//
//         /* Modern Intel, as well as 64-bit CPUs use a revision without stepping */
//         case PROCESSOR_ARCHITECTURE_IA64:
//         case PROCESSOR_ARCHITECTURE_AMD64:
//             swprintf(ValueBuffer, L"%04x", ProcessorInfo.ProcessorRevision);
//             break;
//
//         /* And anything else we'll just read the whole revision identifier */
//         default:
//             swprintf(ValueBuffer, L"%u", ProcessorInfo.ProcessorRevision);
//             break;
//     }
//
//     /* Write the revision to the registry */
//     DPRINT("Setting %wZ to %S\n", &ValueName, ValueBuffer);
//     Status = NtSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_SZ,
//                            ValueBuffer,
//                            (ULONG)(wcslen(ValueBuffer) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Failed writing %wZ environment variable - %x\n",
//                 &ValueName, Status);
//         NtClose(KeyHandle);
//         return Status;
//     }
//
//     /* And finally, write the number of CPUs */
//     RtlInitUnicodeString(&ValueName, L"NUMBER_OF_PROCESSORS");
//     swprintf(ValueBuffer, L"%d", BasicInfo.NumberOfProcessors);
//     DPRINT("Setting %wZ to %S\n", &ValueName, ValueBuffer);
//     Status = NtSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_SZ,
//                            ValueBuffer,
//                            (ULONG)(wcslen(ValueBuffer) + 1) * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Failed writing %wZ environment variable - %x\n",
//                 &ValueName, Status);
//         NtClose(KeyHandle);
//         return Status;
//     }
//
//     /* Now we need to write the safeboot option key in a different format */
//     RtlInitUnicodeString(&DestinationString,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\"
//                          L"Control\\Safeboot\\Option");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DestinationString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle2, KEY_ALL_ACCESS, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* This was indeed a safeboot, so check what kind of safeboot it was */
//         RtlInitUnicodeString(&ValueName, L"OptionValue");
//         Status = NtQueryValueKey(KeyHandle2,
//                                  &ValueName,
//                                  KeyValuePartialInformation,
//                                  PartialInfo,
//                                  sizeof(ValueBuffer),
//                                  &ResultLength);
//         NtClose(KeyHandle2);
//         if (NT_SUCCESS(Status) &&
//             (PartialInfo->Type == REG_DWORD) &&
//             (PartialInfo->DataLength >= sizeof(ULONG)))
//         {
//             /* Convert from the integer value to the correct specifier */
//             RtlInitUnicodeString(&ValueName, L"SAFEBOOT_OPTION");
//             switch (*(PULONG)PartialInfo->Data)
//             {
//                 case 1:
//                     wcscpy(ValueBuffer, L"MINIMAL");
//                     break;
//                 case 2:
//                     wcscpy(ValueBuffer, L"NETWORK");
//                     break;
//                 case 3:
//                     wcscpy(ValueBuffer, L"DSREPAIR");
//                     break;
//             }
//
//             /* And write it in the environment! */
//             DPRINT("Setting %wZ to %S\n", &ValueName, ValueBuffer);
//             Status = NtSetValueKey(KeyHandle,
//                                    &ValueName,
//                                    0,
//                                    REG_SZ,
//                                    ValueBuffer,
//                                    (ULONG)(wcslen(ValueBuffer) + 1) * sizeof(WCHAR));
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("SMSS: Failed writing %wZ environment variable - %x\n",
//                         &ValueName, Status);
//                 NtClose(KeyHandle);
//                 return Status;
//             }
//         }
//         else
//         {
//             DPRINT1("SMSS: Failed to query SAFEBOOT option (Type %lu, Status 0x%x)\n",
//                     PartialInfo->Type, Status);
//         }
//     }
//
//     /* We are all done now */
//     NtClose(KeyHandle);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpProcessFileRenames(VOID)
// {
//     BOOLEAN OldState, HavePrivilege = FALSE;
//     NTSTATUS Status;
//     HANDLE FileHandle, OtherFileHandle;
//     FILE_INFORMATION_CLASS InformationClass;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING FileString;
//     FILE_BASIC_INFORMATION BasicInfo;
//     FILE_DISPOSITION_INFORMATION DeleteInformation;
//     PFILE_RENAME_INFORMATION Buffer;
//     PLIST_ENTRY Head, NextEntry;
//     PSMP_REGISTRY_VALUE RegEntry;
//     PWCHAR FileName;
//     ULONG ValueLength, Length;
//
//     /* Give us access to restore any files we want */
//     Status = RtlAdjustPrivilege(SE_RESTORE_PRIVILEGE, TRUE, FALSE, &OldState);
//     if (NT_SUCCESS(Status)) HavePrivilege = TRUE;
//
//     // FIXME: Handle SFC-protected file renames!
//     if (SmpAllowProtectedRenames)
//         DPRINT1("SMSS: FIXME: Handle SFC-protected file renames!\n");
//
//     /* Process pending files to rename */
//     Head = &SmpFileRenameList;
//     while (!IsListEmpty(Head))
//     {
//         /* Get this entry */
//         NextEntry = RemoveHeadList(Head);
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//         DPRINT("Processing PFRO: '%wZ' / '%wZ'\n", &RegEntry->Value, &RegEntry->Name);
//
//         /* Skip past the '@' marker */
//         if (!(RegEntry->Value.Length) && (*RegEntry->Name.Buffer == L'@'))
//         {
//             RegEntry->Name.Length -= sizeof(UNICODE_NULL);
//             RegEntry->Name.Buffer++;
//         }
//
//         /* Open the file for delete access */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &RegEntry->Name,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//         Status = NtOpenFile(&OtherFileHandle,
//                             DELETE | SYNCHRONIZE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE,
//                             FILE_SYNCHRONOUS_IO_NONALERT);
//         if (!NT_SUCCESS(Status)) goto Quickie;
//
//         /* Check if it's a rename or just a delete */
//         ValueLength = RegEntry->Value.Length;
//         if (!ValueLength)
//         {
//             /* Just a delete, set up the class, length and buffer */
//             InformationClass = FileDispositionInformation;
//             Length = sizeof(DeleteInformation);
//             Buffer = (PFILE_RENAME_INFORMATION)&DeleteInformation;
//
//             /* Set the delete disposition */
//             DeleteInformation.DeleteFileA = TRUE;
//         }
//         else
//         {
//             /* This is a rename, setup the class and length */
//             InformationClass = FileRenameInformation;
//             Length = ValueLength + sizeof(FILE_RENAME_INFORMATION);
//
//             /* Skip past the special markers */
//             FileName = RegEntry->Value.Buffer;
//             if ((*FileName == L'!') || (*FileName == L'@'))
//             {
//                 FileName++;
//                 Length -= sizeof(UNICODE_NULL);
//             }
//
//             /* Now allocate the buffer for the rename information */
//             Buffer = RtlAllocateHeap(RtlGetProcessHeap(), SmBaseTag, Length);
//             if (Buffer)
//             {
//                 /* Setup the buffer to point to the filename, and copy it */
//                 Buffer->RootDirectory = NULL;
//                 Buffer->FileNameLength = Length - sizeof(FILE_RENAME_INFORMATION);
//                 Buffer->ReplaceIfExists = FileName != RegEntry->Value.Buffer;
//                 RtlCopyMemory(Buffer->FileName, FileName, Buffer->FileNameLength);
//             }
//             else
//             {
//                 /* Fail */
//                 Status = STATUS_NO_MEMORY;
//             }
//         }
//
//         /* Check if everything is okay till here */
//         if (NT_SUCCESS(Status))
//         {
//             /* Now either rename or delete the file as requested */
//             Status = NtSetInformationFile(OtherFileHandle,
//                                           &IoStatusBlock,
//                                           Buffer,
//                                           Length,
//                                           InformationClass);
//
//             /* Check if we seem to have failed because the file was readonly */
//             if (!NT_SUCCESS(Status) &&
//                 (InformationClass == FileRenameInformation) &&
//                 (Status == STATUS_OBJECT_NAME_COLLISION) &&
//                 Buffer->ReplaceIfExists)
//             {
//                 /* Open the file for write attribute access this time... */
//                 DPRINT("\nSMSS: '%wZ' => '%wZ' failed - Status == %x, Possible readonly target\n",
//                         &RegEntry->Name,
//                         &RegEntry->Value,
//                         STATUS_OBJECT_NAME_COLLISION);
//                 FileString.Length = RegEntry->Value.Length - sizeof(WCHAR);
//                 FileString.MaximumLength = RegEntry->Value.MaximumLength - sizeof(WCHAR);
//                 FileString.Buffer = FileName;
//                 InitializeObjectAttributes(&ObjectAttributes,
//                                            &FileString,
//                                            OBJ_CASE_INSENSITIVE,
//                                            NULL,
//                                            NULL);
//                 Status = NtOpenFile(&FileHandle,
//                                     FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
//                                     &ObjectAttributes,
//                                     &IoStatusBlock,
//                                     FILE_SHARE_READ | FILE_SHARE_WRITE,
//                                     FILE_SYNCHRONOUS_IO_NONALERT);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* That didn't work, so bail out */
//                     DPRINT1("     SMSS: Open Existing file Failed - Status == %x\n",
//                             Status);
//                 }
//                 else
//                 {
//                     /* Now remove the read-only attribute from the file */
//                     DPRINT("     SMSS: Open Existing Success\n");
//                     RtlZeroMemory(&BasicInfo, sizeof(BasicInfo));
//                     BasicInfo.FileAttributes = FILE_ATTRIBUTE_NORMAL;
//                     Status = NtSetInformationFile(FileHandle,
//                                                   &IoStatusBlock,
//                                                   &BasicInfo,
//                                                   sizeof(BasicInfo),
//                                                   FileBasicInformation);
//                     NtClose(FileHandle);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         /* That didn't work, bail out */
//                         DPRINT1("     SMSS: Set To NORMAL Failed - Status == %x\n",
//                                 Status);
//                     }
//                     else
//                     {
//                         /* Now that the file is no longer read-only, delete! */
//                         DPRINT("     SMSS: Set To NORMAL OK\n");
//                         Status = NtSetInformationFile(OtherFileHandle,
//                                                       &IoStatusBlock,
//                                                       Buffer,
//                                                       Length,
//                                                       FileRenameInformation);
//                         if (!NT_SUCCESS(Status))
//                         {
//                             /* That failed too! */
//                             DPRINT1("     SMSS: Re-Rename Failed - Status == %x\n",
//                                     Status);
//                         }
//                         else
//                         {
//                             /* Everything ok */
//                             DPRINT("     SMSS: Re-Rename Worked OK\n");
//                         }
//                     }
//                 }
//             }
//         }
//
//         /* Close the file handle and check the operation result */
//         NtClose(OtherFileHandle);
// Quickie:
//         if (!NT_SUCCESS(Status))
//         {
//             /* We totally failed */
//             DPRINT1("SMSS: '%wZ' => '%wZ' failed - Status == %x\n",
//                     &RegEntry->Name, &RegEntry->Value, Status);
//         }
//         else if (RegEntry->Value.Length)
//         {
//             /* We succeed with a rename */
//             DPRINT("SMSS: '%wZ' (renamed to) '%wZ'\n", &RegEntry->Name, &RegEntry->Value);
//         }
//         else
//         {
//             /* We succeeded with a delete */
//             DPRINT("SMSS: '%wZ' (deleted)\n", &RegEntry->Name);
//         }
//
//         /* Now free this entry and keep going */
//         if (RegEntry->AnsiValue) RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->AnsiValue);
//         if (RegEntry->Value.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->Value.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry);
//     }
//
//     /* Put back the restore privilege if we had requested it, and return */
//     if (HavePrivilege) RtlAdjustPrivilege(SE_RESTORE_PRIVILEGE, FALSE, FALSE, &OldState);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpLoadDataFromRegistry(OUT PUNICODE_STRING InitialCommand)
// {
//     NTSTATUS Status;
//     PLIST_ENTRY Head, NextEntry;
//     PSMP_REGISTRY_VALUE RegEntry;
//     PVOID OriginalEnvironment;
//     ULONG MuSessionId = 0;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle;
//     UNICODE_STRING DestinationString;
//
//     /* Initialize the keywords we'll be looking for */
//     RtlInitUnicodeString(&SmpDebugKeyword, L"debug");
//     RtlInitUnicodeString(&SmpASyncKeyword, L"async");
//     RtlInitUnicodeString(&SmpAutoChkKeyword, L"autocheck");
//
//     /* Initialize all the registry-associated list heads */
//     InitializeListHead(&SmpBootExecuteList);
//     InitializeListHead(&SmpSetupExecuteList);
//     InitializeListHead(&SmpPagingFileList);
//     InitializeListHead(&SmpDosDevicesList);
//     InitializeListHead(&SmpFileRenameList);
//     InitializeListHead(&SmpKnownDllsList);
//     InitializeListHead(&SmpExcludeKnownDllsList);
//     InitializeListHead(&SmpSubSystemList);
//     InitializeListHead(&SmpSubSystemsToLoad);
//     InitializeListHead(&SmpSubSystemsToDefer);
//     InitializeListHead(&SmpExecuteList);
//
//     SmpPagingFileInitialize();
//
//     /* Initialize the SMSS environment */
//     Status = RtlCreateEnvironment(TRUE, &SmpDefaultEnvironment);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail if there was a problem */
//         DPRINT1("SMSS: Unable to allocate default environment - Status == %X\n",
//                 Status);
//         SMSS_CHECKPOINT(RtlCreateEnvironment, Status);
//         return Status;
//     }
//
//     /* Check if we were booted in PE mode (LiveCD should have this) */
//     RtlInitUnicodeString(&DestinationString,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\"
//                          L"Control\\MiniNT");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DestinationString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle, KEY_ALL_ACCESS, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* If the key exists, we were */
//         NtClose(KeyHandle);
//         MiniNTBoot = TRUE;
//     }
//
//     /* Print out if this is the case */
//     if (MiniNTBoot) DPRINT("SMSS: !!! MiniNT Boot !!!\n");
//
//     /* Open the environment key to see if we are booted in safe mode */
//     RtlInitUnicodeString(&DestinationString,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\"
//                          L"Control\\Session Manager\\Environment");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DestinationString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle, KEY_ALL_ACCESS, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* Delete the value if we found it */
//         RtlInitUnicodeString(&DestinationString, L"SAFEBOOT_OPTION");
//         NtDeleteValueKey(KeyHandle, &DestinationString);
//         NtClose(KeyHandle);
//     }
//
//     /* Switch environments, then query the registry for all needed settings */
//     OriginalEnvironment = NtCurrentPeb()->ProcessParameters->Environment;
//     NtCurrentPeb()->ProcessParameters->Environment = SmpDefaultEnvironment;
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_CONTROL,
//                                     L"Session Manager",
//                                     SmpRegistryConfigurationTable,
//                                     NULL,
//                                     NULL);
//     SmpDefaultEnvironment = NtCurrentPeb()->ProcessParameters->Environment;
//     NtCurrentPeb()->ProcessParameters->Environment = OriginalEnvironment;
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed somewhere in registry initialization, which is bad... */
//         DPRINT1("SMSS: RtlQueryRegistryValues failed - Status == %lx\n", Status);
//         SMSS_CHECKPOINT(RtlQueryRegistryValues, Status);
//         return Status;
//     }
//
//     /* Now we can start acting on the registry settings. First to DOS devices */
//     Status = SmpInitializeDosDevices();
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         DPRINT1("SMSS: Unable to initialize DosDevices configuration - Status == %lx\n",
//                 Status);
//         SMSS_CHECKPOINT(SmpInitializeDosDevices, Status);
//         return Status;
//     }
//
//     /* Next create the session directory... */
//     RtlInitUnicodeString(&DestinationString, L"\\Sessions");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DestinationString,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_PERMANENT,
//                                NULL,
//                                SmpPrimarySecurityDescriptor);
//     Status = NtCreateDirectoryObject(&SmpSessionsObjectDirectory,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         DPRINT1("SMSS: Unable to create %wZ object directory - Status == %lx\n",
//                 &DestinationString, Status);
//         SMSS_CHECKPOINT(NtCreateDirectoryObject, Status);
//         return Status;
//     }
//
//     /* Next loop all the boot execute binaries */
//     Head = &SmpBootExecuteList;
//     while (!IsListEmpty(Head))
//     {
//         /* Remove each one from the list */
//         NextEntry = RemoveHeadList(Head);
//
//         /* Execute it */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//         SmpExecuteCommand(&RegEntry->Name, 0, NULL, 0);
//
//         /* And free it */
//         if (RegEntry->AnsiValue) RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->AnsiValue);
//         if (RegEntry->Value.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->Value.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry);
//     }
//
//     /* Now do any pending file rename operations... */
//     if (!MiniNTBoot) SmpProcessFileRenames();
//
//     /* And initialize known DLLs... */
//     Status = SmpInitializeKnownDlls();
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail if that didn't work */
//         DPRINT1("SMSS: Unable to initialize KnownDll configuration - Status == %lx\n",
//                 Status);
//         SMSS_CHECKPOINT(SmpInitializeKnownDlls, Status);
//         return Status;
//     }
//
//     /* Create the needed page files */
//     if (!MiniNTBoot)
//     {
//         /* Loop every page file */
//         Head = &SmpPagingFileList;
//         while (!IsListEmpty(Head))
//         {
//             /* Remove each one from the list */
//             NextEntry = RemoveHeadList(Head);
//
//             /* Create the descriptor for it */
//             RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//             SmpCreatePagingFileDescriptor(&RegEntry->Name);
//
//             /* And free it */
//             if (RegEntry->AnsiValue) RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->AnsiValue);
//             if (RegEntry->Value.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry->Value.Buffer);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RegEntry);
//         }
//
//         /* Now create all the paging files for the descriptors that we have */
//         SmpCreatePagingFiles();
//     }
//
//     /* Tell Cm it's now safe to fully enable write access to the registry */
//     NtInitializeRegistry(CM_BOOT_FLAG_SMSS);
//
//     /* Create all the system-based environment variables for later inheriting */
//     Status = SmpCreateDynamicEnvironmentVariables();
//     if (!NT_SUCCESS(Status))
//     {
//         /* Handle failure */
//         SMSS_CHECKPOINT(SmpCreateDynamicEnvironmentVariables, Status);
//         return Status;
//     }
//
//     /* And finally load all the subsystems for our first session! */
//     Status = SmpLoadSubSystemsForMuSession(&MuSessionId,
//                                            &SmpWindowsSubSysProcessId,
//                                            InitialCommand);
//     ASSERT(MuSessionId == 0);
//     if (!NT_SUCCESS(Status)) SMSS_CHECKPOINT(SmpLoadSubSystemsForMuSession, Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpInit(IN PUNICODE_STRING InitialCommand,
//         OUT PHANDLE ProcessHandle)
// {
//     NTSTATUS Status, Status2;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING PortName, EventName;
//     HANDLE EventHandle, PortHandle;
//     ULONG HardErrorMode;
//
//     /* Create the SMSS Heap */
//     SmBaseTag = RtlCreateTagHeap(RtlGetProcessHeap(),
//                                  0,
//                                  L"SMSS!",
//                                  L"INIT");
//     SmpHeap = RtlGetProcessHeap();
//
//     /* Enable hard errors */
//     HardErrorMode = TRUE;
//     NtSetInformationProcess(NtCurrentProcess(),
//                             ProcessDefaultHardErrorMode,
//                             &HardErrorMode,
//                             sizeof(HardErrorMode));
//
//     /* Initialize the subsystem list and the session list, plus their locks */
//     RtlInitializeCriticalSection(&SmpKnownSubSysLock);
//     InitializeListHead(&SmpKnownSubSysHead);
//     RtlInitializeCriticalSection(&SmpSessionListLock);
//     InitializeListHead(&SmpSessionListHead);
//
//     /* Initialize the process list */
//     InitializeListHead(&NativeProcessList);
//
//     /* Initialize session parameters */
//     SmpNextSessionId = 1;
//     SmpNextSessionIdScanMode = FALSE;
//     SmpDbgSsLoaded = FALSE;
//
//     /* Create the initial security descriptors */
//     Status = SmpCreateSecurityDescriptors(TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         SMSS_CHECKPOINT(SmpCreateSecurityDescriptors, Status);
//         return Status;
//     }
//
//     /* Initialize subsystem names */
//     RtlInitUnicodeString(&SmpSubsystemName, L"NT-Session Manager");
//     RtlInitUnicodeString(&PosixName, L"POSIX");
//     RtlInitUnicodeString(&Os2Name, L"OS2");
//
//     /* Create the SM API Port */
//     RtlInitUnicodeString(&PortName, L"\\SmApiPort");
//     InitializeObjectAttributes(&ObjectAttributes, &PortName, 0, NULL, SmpApiPortSecurityDescriptor);
//     Status = NtCreatePort(&PortHandle,
//                           &ObjectAttributes,
//                           sizeof(SB_CONNECTION_INFO),
//                           sizeof(SM_API_MSG),
//                           sizeof(SB_API_MSG) * 32);
//     ASSERT(NT_SUCCESS(Status));
//     SmpDebugPort = PortHandle;
//
//     /* Create two SM API threads */
//     Status = RtlCreateUserThread(NtCurrentProcess(),
//                                  NULL,
//                                  FALSE,
//                                  0,
//                                  0,
//                                  0,
//                                  SmpApiLoop,
//                                  PortHandle,
//                                  NULL,
//                                  NULL);
//     ASSERT(NT_SUCCESS(Status));
//     Status = RtlCreateUserThread(NtCurrentProcess(),
//                                  NULL,
//                                  FALSE,
//                                  0,
//                                  0,
//                                  0,
//                                  SmpApiLoop,
//                                  PortHandle,
//                                  NULL,
//                                  NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Create the write event that autochk can set after running */
//     RtlInitUnicodeString(&EventName, L"\\Device\\VolumesSafeForWriteAccess");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &EventName,
//                                OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//     Status2 = NtCreateEvent(&EventHandle,
//                             EVENT_ALL_ACCESS,
//                             &ObjectAttributes,
//                             0,
//                             0);
//     if (!NT_SUCCESS(Status2))
//     {
//         /* Should never really fail */
//         DPRINT1("SMSS: Unable to create %wZ event - Status == %lx\n",
//                 &EventName, Status2);
//         ASSERT(NT_SUCCESS(Status2));
//     }
//
//     /* Now initialize everything else based on the registry parameters */
//     Status = SmpLoadDataFromRegistry(InitialCommand);
//     if (NT_SUCCESS(Status))
//     {
//         /* Autochk should've run now. Set the event and save the CSRSS handle */
//         *ProcessHandle = SmpWindowsSubSysProcess;
//
//         /* If no Windows subsystem was started (e.g. ARM64 without Win32k/CSRSS),
//          * create a dummy unsignaled event so NtWaitForMultipleObjects doesn't
//          * fail with STATUS_INVALID_HANDLE on the CSRSS handle slot. */
//         if (!*ProcessHandle)
//         {
//             NtCreateEvent(ProcessHandle, EVENT_ALL_ACCESS, NULL, NotificationEvent, FALSE);
//         }
//
//         NtSetEvent(EventHandle, NULL);
//         NtClose(EventHandle);
//     }
//
//     /* All done */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
