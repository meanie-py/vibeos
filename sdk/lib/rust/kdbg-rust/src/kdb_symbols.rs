use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::*;

use ntkrnl_sys::ddk::{
    PLDR_DATA_TABLE_ENTRY,
    KEVENT,
    _EPROCESS,
    _OBJECT_ATTRIBUTES,
    _IO_STATUS_BLOCK,
    _UNICODE_STRING,
    PUNICODE_STRING,
    PCONTEXT,
    PSTR,
    INT,
    KeInitializeEvent,
    KeInitializeSpinLock,
    KeWaitForSingleObject,
    MmUnloadSystemImage,
    NtClose,
    PsCreateSystemThread,
    RtlAppendUnicodeStringToString,
    RtlCopyUnicodeString,
    RtlInitEmptyUnicodeString,
    ZwOpenFile,
};

// File-local constants (not exported via ntkrnl-sys)
const FILE_READ_ACCESS: ULONG = 0x00000001;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;

// Sync wrapper to allow `static` items containing raw pointers
#[repr(transparent)]
struct SyncUnicode(_UNICODE_STRING);
unsafe impl Sync for SyncUnicode {}

#[repr(transparent)]
struct SyncWchars<const N: usize>([u16; N]);
unsafe impl<const N: usize> Sync for SyncWchars<N> {}

extern "system" {
    // NTAPI (stdcall on i386 → `_ExInterlockedRemoveHeadList@8`). Was `extern "C"`
    // (cdecl), which only linked while rustc happened to inline this intra-crate
    // call; a CGU repartition routed it through the symbol and exposed the
    // mismatch on i386. The definition (ex/interlocked.rs) is `extern "system"`.
    fn ExInterlockedRemoveHeadList(
        ListHead: PLIST_ENTRY,
        Lock: PKSPIN_LOCK,
    ) -> PLIST_ENTRY;
}

extern "C" {
    fn CONTAINING_RECORD(address: PVOID, type_size: usize, field_offset: usize) -> PVOID; // ntoskrnl shim_exports
    fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: PUNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID, );
    static mut PsLoadedModuleSpinLock: KSPIN_LOCK;
    static mut PsLoadedModuleList: LIST_ENTRY;
    static mut KeLoaderBlock: *mut core::ffi::c_void;
    fn RosSymGetAddressInformation( info: PVOID, address: ULONG_PTR, line_number: *mut ULONG, file_name: PSTR, function_name: PSTR, ) -> BOOLEAN;
    fn RosSymCreateFromFile(file_handle: *mut HANDLE, info: *mut PVOID) -> BOOLEAN;
    fn RosSymDelete(info: PVOID);
    fn RosSymCreateFromMem(base: PVOID, size: ULONG, info: *mut PVOID) -> BOOLEAN;
    fn RosSymInitKernelMode();
    fn _strnicmp(s1: PSTR, s2: PSTR, n: usize) -> i32;
    fn isspace(c: i32) -> i32;
    fn toupper(c: i32) -> i32;
}

#[repr(C)]
struct IMAGE_SYMBOL_INFO_CACHE {
    ListEntry: LIST_ENTRY,
    RefCount: ULONG,
    FileName: _UNICODE_STRING,
    RosSymInfo: PVOID,
}

type PIMAGE_SYMBOL_INFO_CACHE = *mut IMAGE_SYMBOL_INFO_CACHE;

static mut LoadSymbols: BOOLEAN = 0u8;
static mut SymbolsToLoad: LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut SymbolsToLoadLock: KSPIN_LOCK = unsafe { core::mem::zeroed() };
static mut SymbolsToLoadEvent: KEVENT = unsafe { core::mem::zeroed() };

unsafe fn KdbpSymSearchModuleList(
    mut current_entry: PLIST_ENTRY,
    end_entry: PLIST_ENTRY,
    Count: *mut LONG,
    Address: PVOID,
    Index: INT,
    pLdrEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> BOOLEAN {
    while !current_entry.is_null() && current_entry != end_entry {
        *pLdrEntry = CONTAINING_RECORD(
            current_entry as PVOID,
            core::mem::size_of::<ntkrnl_sys::ddk::_LDR_DATA_TABLE_ENTRY>(),
            core::mem::offset_of!(ntkrnl_sys::ddk::_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks),
        ) as PLDR_DATA_TABLE_ENTRY;

        if (!Address.is_null()
            && Address >= (*(*pLdrEntry)).DllBase
            && (Address as ULONG_PTR) < ((*(*pLdrEntry)).DllBase as ULONG_PTR + (*(*pLdrEntry)).SizeOfImage as ULONG_PTR))
            || (Index >= 0 && { *Count += 1; *Count - 1 } == Index)
        {
            return 1u8;
        }

        current_entry = (*current_entry).Flink;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpSymFindModule(
    Address: PVOID,
    Index: INT,
    pLdrEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> BOOLEAN {
    let mut Count: LONG = 0;
    let CurrentProcess: *mut _EPROCESS;

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(PsLoadedModuleSpinLock));
    if KdbpSymSearchModuleList(
        PsLoadedModuleList.Flink,
        core::ptr::addr_of_mut!(PsLoadedModuleList),
        core::ptr::addr_of_mut!(Count),
        Address,
        Index,
        pLdrEntry,
    ) != 0
    {
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!(PsLoadedModuleSpinLock));
        return 1u8;
    }
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!(PsLoadedModuleSpinLock));

    CurrentProcess = PsGetCurrentProcess() as *mut _EPROCESS;

    if CurrentProcess.is_null()
        || (*CurrentProcess).Peb.is_null()
        || (*(*CurrentProcess).Peb).Ldr.is_null()
    {
        return 0u8;
    }

    KdbpSymSearchModuleList(
        (*(*(*CurrentProcess).Peb).Ldr).InLoadOrderModuleList.Flink,
        core::ptr::addr_of_mut!((*(*(*CurrentProcess).Peb).Ldr).InLoadOrderModuleList),
        core::ptr::addr_of_mut!(Count),
        Address,
        Index,
        pLdrEntry,
    )
}

unsafe fn KdbpSymUnicodeToAnsi(
    Unicode: PUNICODE_STRING,
    Ansi: PSTR,
    Length: ULONG,
) -> PSTR {
    let mut p: PSTR;
    let mut pw: *mut WCHAR;
    let mut i: ULONG;

    i = (*Unicode).Length as ULONG / core::mem::size_of::<WCHAR>() as ULONG;
    i = core::cmp::min(i, Length - 1);

    pw = (*Unicode).Buffer;
    p = Ansi;
    while i > 0 {
        *p = (*pw) as CHAR;
        p = p.add(1);
        pw = pw.add(1);
        i -= 1;
    }

    *p = 0i8;
    Ansi
}

#[no_mangle]
pub unsafe extern "C" fn KdbSymPrintAddress(
    Address: PVOID,
    Context: PCONTEXT,
) -> BOOLEAN {
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let RelativeAddress: ULONG_PTR;
    let mut Printed: BOOLEAN = 0u8;
    let mut ModuleNameAnsi: [CHAR; 64] = [0; 64];

    if KdbpSymFindModule(Address, -1, core::ptr::addr_of_mut!(LdrEntry)) == 0 {
        return 0u8;
    }

    RelativeAddress = Address as ULONG_PTR - (*LdrEntry).DllBase as ULONG_PTR;

    KdbpSymUnicodeToAnsi(
        core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
        ModuleNameAnsi.as_mut_ptr(),
        core::mem::size_of_val(&ModuleNameAnsi) as ULONG,
    );

    if !(*LdrEntry).PatchInformation.is_null() {
        let mut LineNumber: ULONG = 0;
        let mut FileName: [CHAR; 256] = [0; 256];
        let mut FunctionName: [CHAR; 256] = [0; 256];

        if RosSymGetAddressInformation(
            (*LdrEntry).PatchInformation,
            RelativeAddress,
            core::ptr::addr_of_mut!(LineNumber),
            FileName.as_mut_ptr(),
            FunctionName.as_mut_ptr(),
        ) != 0
        {
            crate::kdb_printf!(b"<%s:%x (%s:%d (%s))>\0",
                ModuleNameAnsi.as_ptr(),
                RelativeAddress,
                FileName.as_ptr(),
                LineNumber,
                FunctionName.as_ptr(),
            );
            Printed = 1u8;
        }
    }

    if Printed == 0 {
        crate::kdb_printf!(b"<%s:%x>\0",
            ModuleNameAnsi.as_ptr(),
            RelativeAddress,
        );
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn LoadSymbolsRoutine(Context: PVOID) {
    loop {
        let Status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!(SymbolsToLoadEvent) as PVOID,
            ntkrnl_sys::ddk::_KWAIT_REASON_WrKernel as _,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            LoadSymbols = 0u8;
            return;
        }

        loop {
            let ListEntry = ExInterlockedRemoveHeadList(
                core::ptr::addr_of_mut!(SymbolsToLoad),
                core::ptr::addr_of_mut!(SymbolsToLoadLock),
            );
            if ListEntry.is_null() {
                break;
            }

            let LdrEntry = CONTAINING_RECORD(
            ListEntry as PVOID,
            core::mem::size_of::<ntkrnl_sys::ddk::_LDR_DATA_TABLE_ENTRY>(),
            core::mem::offset_of!(ntkrnl_sys::ddk::_LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks),
        ) as PLDR_DATA_TABLE_ENTRY;
            let mut FileHandle: HANDLE = core::ptr::null_mut();
            let mut Attrib: _OBJECT_ATTRIBUTES = core::mem::zeroed();
            let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
            InitializeObjectAttributes(
                core::ptr::addr_of_mut!(Attrib),
                core::ptr::addr_of_mut!((*LdrEntry).FullDllName),
                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            let mut Status = ZwOpenFile(
                core::ptr::addr_of_mut!(FileHandle),
                FILE_READ_ACCESS | SYNCHRONIZE,
                core::ptr::addr_of_mut!(Attrib),
                core::ptr::addr_of_mut!(Iosb),
                FILE_SHARE_READ,
                FILE_SYNCHRONOUS_IO_NONALERT,
            );
            if Status < 0 {
                static System32Dir: SyncWchars<24> = SyncWchars([
                    b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16,
                    b'm' as u16, b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, b'\\' as u16,
                    b's' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
                    b'3' as u16, b'2' as u16, b'\\' as u16, 0, 0, 0,
                ]);
                static System32DirUnicode: SyncUnicode = SyncUnicode(_UNICODE_STRING {
                    Length: 22 * 2,
                    MaximumLength: 24 * 2,
                    Buffer: System32Dir.0.as_ptr() as *mut u16,
                });
                let mut ImagePath: _UNICODE_STRING = core::mem::zeroed();
                let mut ImagePathBuffer: [WCHAR; 256] = [0; 256];
                RtlInitEmptyUnicodeString(
                    core::ptr::addr_of_mut!(ImagePath),
                    ImagePathBuffer.as_mut_ptr(),
                    core::mem::size_of_val(&ImagePathBuffer) as u16,
                );
                RtlCopyUnicodeString(
                    core::ptr::addr_of_mut!(ImagePath),
                    core::ptr::addr_of!(System32DirUnicode.0) as *mut _UNICODE_STRING,
                );
                RtlAppendUnicodeStringToString(
                    core::ptr::addr_of_mut!(ImagePath),
                    core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                );
                InitializeObjectAttributes(
                    core::ptr::addr_of_mut!(Attrib),
                    core::ptr::addr_of_mut!(ImagePath),
                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                Status = ZwOpenFile(
                    core::ptr::addr_of_mut!(FileHandle),
                    FILE_READ_ACCESS | SYNCHRONIZE,
                    core::ptr::addr_of_mut!(Attrib),
                    core::ptr::addr_of_mut!(Iosb),
                    FILE_SHARE_READ,
                    FILE_SYNCHRONOUS_IO_NONALERT,
                );
                if Status < 0 {
                    static DriversDir: SyncWchars<33> = SyncWchars([
                        b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16,
                        b'm' as u16, b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, b'\\' as u16,
                        b's' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
                        b'3' as u16, b'2' as u16, b'\\' as u16, b'd' as u16, b'r' as u16, b'i' as u16,
                        b'v' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'\\' as u16, 0, 0, 0, 0,
                    ]);
                    static DriversDirUnicode: SyncUnicode = SyncUnicode(_UNICODE_STRING {
                        Length: 29 * 2,
                        MaximumLength: 33 * 2,
                        Buffer: DriversDir.0.as_ptr() as *mut u16,
                    });

                    RtlInitEmptyUnicodeString(
                        core::ptr::addr_of_mut!(ImagePath),
                        ImagePathBuffer.as_mut_ptr(),
                        core::mem::size_of_val(&ImagePathBuffer) as u16,
                    );
                    RtlCopyUnicodeString(
                        core::ptr::addr_of_mut!(ImagePath),
                        core::ptr::addr_of!(DriversDirUnicode.0) as *mut _UNICODE_STRING,
                    );
                    RtlAppendUnicodeStringToString(
                        core::ptr::addr_of_mut!(ImagePath),
                        core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                    );
                    InitializeObjectAttributes(
                        core::ptr::addr_of_mut!(Attrib),
                        core::ptr::addr_of_mut!(ImagePath),
                        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    Status = ZwOpenFile(
                        core::ptr::addr_of_mut!(FileHandle),
                        FILE_READ_ACCESS | SYNCHRONIZE,
                        core::ptr::addr_of_mut!(Attrib),
                        core::ptr::addr_of_mut!(Iosb),
                        FILE_SHARE_READ,
                        FILE_SYNCHRONOUS_IO_NONALERT,
                    );
                }
            }

            if Status < 0 {
                MmUnloadSystemImage(LdrEntry as PVOID);
                continue;
            }

            if RosSymCreateFromFile(
                core::ptr::addr_of_mut!(FileHandle),
                core::ptr::addr_of_mut!((*LdrEntry).PatchInformation),
            ) == 0
            {
                (*LdrEntry).PatchInformation = core::ptr::null_mut();
            }

            NtClose(FileHandle);
            MmUnloadSystemImage(LdrEntry as PVOID);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn KdbSymProcessSymbols(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    Load: BOOLEAN,
) {
    if LoadSymbols == 0 {
        return;
    }

    if Load == 0 {
        if !(*LdrEntry).PatchInformation.is_null() {
            RosSymDelete((*LdrEntry).PatchInformation);
            (*LdrEntry).PatchInformation = core::ptr::null_mut();
        }
        return;
    }

    if RosSymCreateFromMem(
        (*LdrEntry).DllBase,
        (*LdrEntry).SizeOfImage,
        core::ptr::addr_of_mut!((*LdrEntry).PatchInformation),
    ) != 0
    {
        return;
    }

    (*LdrEntry).LoadCount += 1;

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(SymbolsToLoadLock));
    InsertTailList(
        core::ptr::addr_of_mut!(SymbolsToLoad),
        core::ptr::addr_of_mut!((*LdrEntry).InInitializationOrderLinks) as PLIST_ENTRY,
    );
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!(SymbolsToLoadLock));

    KeSetEvent(
        core::ptr::addr_of_mut!(SymbolsToLoadEvent) as *mut _,
        0i32,
        0u8,
    );
}

#[no_mangle]
pub unsafe extern "C" fn KdbSymInit(BootPhase: ULONG) -> BOOLEAN {
    #[used]
    static mut OrigLoadSymbols: BOOLEAN = 0u8;

    if BootPhase == 0 {
        let mut CommandLine: PSTR;
        let mut Found: i16 = 0;
        let mut YesNo: CHAR;

        #[cfg(all(debug_assertions, target_arch = "x86"))]
        {
            LoadSymbols = 1u8;
        }
        #[cfg(not(all(debug_assertions, target_arch = "x86")))]
        {
            LoadSymbols = 0u8;
        }

        CommandLine = *(KeLoaderBlock as *mut PSTR).add(8 / core::mem::size_of::<PSTR>());
        while *CommandLine != 0 {
            while isspace(*CommandLine as i32) != 0 {
                CommandLine = CommandLine.add(1);
            }

            Found = 0;
            if _strnicmp(CommandLine, b"LOADSYMBOLS\0".as_ptr() as PSTR, 11) == 0 {
                Found = 1;
                CommandLine = CommandLine.add(11);
            } else if _strnicmp(CommandLine, b"NOLOADSYMBOLS\0".as_ptr() as PSTR, 13) == 0 {
                Found = -1;
                CommandLine = CommandLine.add(13);
            }
            if Found != 0 {
                if *CommandLine == b'=' as i8 {
                    CommandLine = CommandLine.add(1);
                    YesNo = toupper(*CommandLine as i32) as CHAR;
                    if YesNo == b'N' as i8 || YesNo == b'0' as i8 {
                        Found = -1 * Found;
                    }
                }
                LoadSymbols = if Found > 0 { 1u8 } else { 0u8 };
            }

            while *CommandLine != 0 && isspace(*CommandLine as i32) == 0 {
                CommandLine = CommandLine.add(1);
            }
        }

        OrigLoadSymbols = LoadSymbols;
        LoadSymbols = 0u8;
        return OrigLoadSymbols;
    } else if BootPhase == 1 {
        let mut Thread: HANDLE = core::ptr::null_mut();
        let Status: NTSTATUS;
        let mut OldIrql: KIRQL = 0;
        let mut ListEntry: PLIST_ENTRY;

        LoadSymbols = OrigLoadSymbols;

        if MmNumberOfPhysicalPages < (96 * 1024 * 1024 / PAGE_SIZE) {
            LoadSymbols = 0u8;
        }

        if LoadSymbols == 0 {
            return LoadSymbols;
        }

        InitializeListHead(core::ptr::addr_of_mut!(SymbolsToLoad));
        KeInitializeSpinLock(core::ptr::addr_of_mut!(SymbolsToLoadLock) as *mut _);
        KeInitializeEvent(
            core::ptr::addr_of_mut!(SymbolsToLoadEvent) as *mut _,
            1u32, // SynchronizationEvent
            0u8,
        );

        Status = PsCreateSystemThread(
            core::ptr::addr_of_mut!(Thread),
            THREAD_ALL_ACCESS,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::mem::transmute(LoadSymbolsRoutine as unsafe extern "system" fn(PVOID)),
            core::ptr::null_mut(),
        );
        if Status < 0 {
            LoadSymbols = 0u8;
            return LoadSymbols;
        }

        RosSymInitKernelMode();

        KeAcquireSpinLock(
            core::ptr::addr_of_mut!(PsLoadedModuleSpinLock),
            core::ptr::addr_of_mut!(OldIrql),
        );

        ListEntry = PsLoadedModuleList.Flink;
        while ListEntry != core::ptr::addr_of_mut!(PsLoadedModuleList) {
            let LdrEntry = CONTAINING_RECORD(
            ListEntry as PVOID,
            core::mem::size_of::<ntkrnl_sys::ddk::_LDR_DATA_TABLE_ENTRY>(),
            core::mem::offset_of!(ntkrnl_sys::ddk::_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks),
        ) as PLDR_DATA_TABLE_ENTRY;
            KdbSymProcessSymbols(LdrEntry, 1u8);
            ListEntry = (*ListEntry).Flink;
        }

        KeReleaseSpinLock(
            core::ptr::addr_of_mut!(PsLoadedModuleSpinLock),
            OldIrql,
        );
    }

    LoadSymbols
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/kdbg/kdb_symbols.c
//  * PURPOSE:         Getting symbol information...
//  *
//  * PROGRAMMERS:     David Welch (welch@cwcom.net)
//  *                  Colin Finck (colin@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #include "kdb.h"
//
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS ******************************************************************/
//
// typedef struct _IMAGE_SYMBOL_INFO_CACHE
// {
//     LIST_ENTRY ListEntry;
//     ULONG RefCount;
//     UNICODE_STRING FileName;
//     PROSSYM_INFO RosSymInfo;
// }
// IMAGE_SYMBOL_INFO_CACHE, *PIMAGE_SYMBOL_INFO_CACHE;
//
// static BOOLEAN LoadSymbols = FALSE;
// static LIST_ENTRY SymbolsToLoad;
// static KSPIN_LOCK SymbolsToLoadLock;
// static KEVENT SymbolsToLoadEvent;
//
// /* FUNCTIONS ****************************************************************/
//
// static
// BOOLEAN
// KdbpSymSearchModuleList(
//     IN PLIST_ENTRY current_entry,
//     IN PLIST_ENTRY end_entry,
//     IN PLONG Count,
//     IN PVOID Address,
//     IN INT Index,
//     OUT PLDR_DATA_TABLE_ENTRY* pLdrEntry)
// {
//     while (current_entry && current_entry != end_entry)
//     {
//         *pLdrEntry = CONTAINING_RECORD(current_entry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
//
//         if ((Address && Address >= (PVOID)(*pLdrEntry)->DllBase && Address < (PVOID)((ULONG_PTR)(*pLdrEntry)->DllBase + (*pLdrEntry)->SizeOfImage)) ||
//             (Index >= 0 && (*Count)++ == Index))
//         {
//             return TRUE;
//         }
//
//         current_entry = current_entry->Flink;
//     }
//
//     return FALSE;
// }
//
// /*! \brief Find a module...
//  *
//  * \param Address      If \a Address is not NULL the module containing \a Address
//  *                     is searched.
//  * \param Name         If \a Name is not NULL the module named \a Name will be
//  *                     searched.
//  * \param Index        If \a Index is >= 0 the Index'th module will be returned.
//  * \param pLdrEntry    Pointer to a PLDR_DATA_TABLE_ENTRY which is filled.
//  *
//  * \retval TRUE    Module was found, \a pLdrEntry was filled.
//  * \retval FALSE   No module was found.
//  */
// BOOLEAN
// KdbpSymFindModule(
//     IN PVOID Address  OPTIONAL,
//     IN INT Index  OPTIONAL,
//     OUT PLDR_DATA_TABLE_ENTRY* pLdrEntry)
// {
//     LONG Count = 0;
//     PEPROCESS CurrentProcess;
//
//     /* First try to look up the module in the kernel module list. */
//     KeAcquireSpinLockAtDpcLevel(&PsLoadedModuleSpinLock);
//     if(KdbpSymSearchModuleList(PsLoadedModuleList.Flink,
//                                &PsLoadedModuleList,
//                                &Count,
//                                Address,
//                                Index,
//                                pLdrEntry))
//     {
//         KeReleaseSpinLockFromDpcLevel(&PsLoadedModuleSpinLock);
//         return TRUE;
//     }
//     KeReleaseSpinLockFromDpcLevel(&PsLoadedModuleSpinLock);
//
//     /* That didn't succeed. Try the module list of the current process now. */
//     CurrentProcess = PsGetCurrentProcess();
//
//     if(!CurrentProcess || !CurrentProcess->Peb || !CurrentProcess->Peb->Ldr)
//         return FALSE;
//
//     return KdbpSymSearchModuleList(CurrentProcess->Peb->Ldr->InLoadOrderModuleList.Flink,
//                                    &CurrentProcess->Peb->Ldr->InLoadOrderModuleList,
//                                    &Count,
//                                    Address,
//                                    Index,
//                                    pLdrEntry);
// }
//
// static
// PCHAR
// NTAPI
// KdbpSymUnicodeToAnsi(IN PUNICODE_STRING Unicode,
//                      OUT PCHAR Ansi,
//                      IN ULONG Length)
// {
//     PCHAR p;
//     PWCHAR pw;
//     ULONG i;
//
//     /* Set length and normalize it */
//     i = Unicode->Length / sizeof(WCHAR);
//     i = min(i, Length - 1);
//
//     /* Set source and destination, and copy */
//     pw = Unicode->Buffer;
//     p = Ansi;
//     while (i--) *p++ = (CHAR)*pw++;
//
//     /* Null terminate and return */
//     *p = ANSI_NULL;
//     return Ansi;
// }
//
// /*! \brief Print address...
//  *
//  * Tries to lookup line number, file name and function name for the given
//  * address and prints it.
//  * If no such information is found the address is printed in the format
//  * <module: offset>, otherwise the format will be
//  * <module: offset (filename:linenumber (functionname))>
//  *
//  * \retval TRUE  Module containing \a Address was found, \a Address was printed.
//  * \retval FALSE  No module containing \a Address was found, nothing was printed.
//  */
// BOOLEAN
// KdbSymPrintAddress(
//     IN PVOID Address,
//     IN PCONTEXT Context)
// {
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     ULONG_PTR RelativeAddress;
//     BOOLEAN Printed = FALSE;
//     CHAR ModuleNameAnsi[64];
//
//     if (!KdbpSymFindModule(Address, -1, &LdrEntry))
//         return FALSE;
//
//     RelativeAddress = (ULONG_PTR)Address - (ULONG_PTR)LdrEntry->DllBase;
//
//     KdbpSymUnicodeToAnsi(&LdrEntry->BaseDllName,
//                         ModuleNameAnsi,
//                         sizeof(ModuleNameAnsi));
//
//     if (LdrEntry->PatchInformation)
//     {
//         ULONG LineNumber;
//         CHAR FileName[256];
//         CHAR FunctionName[256];
//
//         if (RosSymGetAddressInformation(LdrEntry->PatchInformation,
//                                         RelativeAddress,
//                                         &LineNumber,
//                                         FileName,
//                                         FunctionName))
//         {
//             KdbPrintf("<%s:%x (%s:%d (%s))>",
//                       ModuleNameAnsi, RelativeAddress,
//                       FileName, LineNumber, FunctionName);
//             Printed = TRUE;
//         }
//     }
//
//     if (!Printed)
//     {
//         /* Just print module & address */
//         KdbPrintf("<%s:%x>", ModuleNameAnsi, RelativeAddress);
//     }
//
//     return TRUE;
// }
//
// static KSTART_ROUTINE LoadSymbolsRoutine;
// /*! \brief          The symbol loader thread routine.
//  *                  This opens the image file for reading and loads the symbols
//  *                  section from there.
//  *
//  * \note            We must do this because KdbSymProcessSymbols is
//  *                  called at high IRQL and we can't set the event from here
//  *
//  * \param Context   Unused
//  */
// _Use_decl_annotations_
// VOID
// NTAPI
// LoadSymbolsRoutine(
//     _In_ PVOID Context)
// {
//     UNREFERENCED_PARAMETER(Context);
//
//     while (TRUE)
//     {
//         PLIST_ENTRY ListEntry;
//         NTSTATUS Status = KeWaitForSingleObject(&SymbolsToLoadEvent, WrKernel, KernelMode, FALSE, NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("KeWaitForSingleObject failed?! 0x%08x\n", Status);
//             LoadSymbols = FALSE;
//             return;
//         }
//
//         while ((ListEntry = ExInterlockedRemoveHeadList(&SymbolsToLoad, &SymbolsToLoadLock)))
//         {
//             PLDR_DATA_TABLE_ENTRY LdrEntry = CONTAINING_RECORD(ListEntry, LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks);
//             HANDLE FileHandle;
//             OBJECT_ATTRIBUTES Attrib;
//             IO_STATUS_BLOCK Iosb;
//             InitializeObjectAttributes(&Attrib, &LdrEntry->FullDllName, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);
//             DPRINT1("Trying %wZ\n", &LdrEntry->FullDllName);
//             Status = ZwOpenFile(&FileHandle,
//                                 FILE_READ_ACCESS | SYNCHRONIZE,
//                                 &Attrib,
//                                 &Iosb,
//                                 FILE_SHARE_READ,
//                                 FILE_SYNCHRONOUS_IO_NONALERT);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Try system paths */
//                 static const UNICODE_STRING System32Dir = RTL_CONSTANT_STRING(L"\\SystemRoot\\system32\\");
//                 UNICODE_STRING ImagePath;
//                 WCHAR ImagePathBuffer[256];
//                 RtlInitEmptyUnicodeString(&ImagePath, ImagePathBuffer, sizeof(ImagePathBuffer));
//                 RtlCopyUnicodeString(&ImagePath, &System32Dir);
//                 RtlAppendUnicodeStringToString(&ImagePath, &LdrEntry->BaseDllName);
//                 InitializeObjectAttributes(&Attrib, &ImagePath, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);
//                 DPRINT1("Trying %wZ\n", &ImagePath);
//                 Status = ZwOpenFile(&FileHandle,
//                                     FILE_READ_ACCESS | SYNCHRONIZE,
//                                     &Attrib,
//                                     &Iosb,
//                                     FILE_SHARE_READ,
//                                     FILE_SYNCHRONOUS_IO_NONALERT);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     static const UNICODE_STRING DriversDir= RTL_CONSTANT_STRING(L"\\SystemRoot\\system32\\drivers\\");
//
//                     RtlInitEmptyUnicodeString(&ImagePath, ImagePathBuffer, sizeof(ImagePathBuffer));
//                     RtlCopyUnicodeString(&ImagePath, &DriversDir);
//                     RtlAppendUnicodeStringToString(&ImagePath, &LdrEntry->BaseDllName);
//                     InitializeObjectAttributes(&Attrib, &ImagePath, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);
//                     DPRINT1("Trying %wZ\n", &ImagePath);
//                     Status = ZwOpenFile(&FileHandle,
//                                         FILE_READ_ACCESS | SYNCHRONIZE,
//                                         &Attrib,
//                                         &Iosb,
//                                         FILE_SHARE_READ,
//                                         FILE_SYNCHRONOUS_IO_NONALERT);
//                 }
//             }
//
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("Failed opening file %wZ (%wZ) for reading symbols (0x%08x)\n", &LdrEntry->FullDllName, &LdrEntry->BaseDllName, Status);
//                 /* We took a ref previously */
//                 MmUnloadSystemImage(LdrEntry);
//                 continue;
//             }
//
//             /* Hand it to Rossym */
//             if (!RosSymCreateFromFile(&FileHandle, (PROSSYM_INFO*)&LdrEntry->PatchInformation))
//                 LdrEntry->PatchInformation = NULL;
//
//             /* We're done for this one. */
//             NtClose(FileHandle);
//             MmUnloadSystemImage(LdrEntry);
//         }
//     }
// }
//
// /*! \brief          Load symbols from image mapping. If this fails,
//  *
//  * \param LdrEntry  The entry to load symbols from
//  */
// VOID
// KdbSymProcessSymbols(
//     _Inout_ PLDR_DATA_TABLE_ENTRY LdrEntry,
//     _In_ BOOLEAN Load)
// {
//     if (!LoadSymbols)
//         return;
//
//     /* Check if this is unload */
//     if (!Load)
//     {
//         /* Did we process it */
//         if (LdrEntry->PatchInformation)
//         {
//             RosSymDelete(LdrEntry->PatchInformation);
//             LdrEntry->PatchInformation = NULL;
//         }
//         return;
//     }
//
//     if (RosSymCreateFromMem(LdrEntry->DllBase, LdrEntry->SizeOfImage, (PROSSYM_INFO*)&LdrEntry->PatchInformation))
//     {
//         return;
//     }
//
//     /* Add a ref until we really process it */
//     LdrEntry->LoadCount++;
//
//     /* Tell our worker thread to read from it */
//     KeAcquireSpinLockAtDpcLevel(&SymbolsToLoadLock);
//     InsertTailList(&SymbolsToLoad, &LdrEntry->InInitializationOrderLinks);
//     KeReleaseSpinLockFromDpcLevel(&SymbolsToLoadLock);
//
//     KeSetEvent(&SymbolsToLoadEvent, IO_NO_INCREMENT, FALSE);
// }
//
//
// /**
//  * @brief   Initializes the KDB symbols implementation.
//  *
//  * @param[in]   BootPhase
//  * Phase of initialization.
//  *
//  * @return
//  * TRUE if symbols are to be loaded at this given BootPhase; FALSE if not.
//  **/
// BOOLEAN
// KdbSymInit(
//     _In_ ULONG BootPhase)
// {
// #if 1 // FIXME: This is a workaround HACK!!
//     static BOOLEAN OrigLoadSymbols = FALSE;
// #endif
//
//     DPRINT("KdbSymInit() BootPhase=%d\n", BootPhase);
//
//     if (BootPhase == 0)
//     {
//         PSTR CommandLine;
//         SHORT Found = FALSE;
//         CHAR YesNo;
//
//         /* By default, load symbols in DBG builds, but not in REL builds
//            or anything other than x86, because they only work on x86
//            and can cause the system to hang on x64. */
// #if DBG && defined(_M_IX86)
//         LoadSymbols = TRUE;
// #else
//         LoadSymbols = FALSE;
// #endif
//
//         /* Check the command line for LOADSYMBOLS, NOLOADSYMBOLS,
//          * LOADSYMBOLS={YES|NO}, NOLOADSYMBOLS={YES|NO} */
//         ASSERT(KeLoaderBlock);
//         CommandLine = KeLoaderBlock->LoadOptions;
//         while (*CommandLine)
//         {
//             /* Skip any whitespace */
//             while (isspace(*CommandLine))
//                 ++CommandLine;
//
//             Found = 0;
//             if (_strnicmp(CommandLine, "LOADSYMBOLS", 11) == 0)
//             {
//                 Found = +1;
//                 CommandLine += 11;
//             }
//             else if (_strnicmp(CommandLine, "NOLOADSYMBOLS", 13) == 0)
//             {
//                 Found = -1;
//                 CommandLine += 13;
//             }
//             if (Found != 0)
//             {
//                 if (*CommandLine == '=')
//                 {
//                     ++CommandLine;
//                     YesNo = toupper(*CommandLine);
//                     if (YesNo == 'N' || YesNo == '0')
//                     {
//                         Found = -1 * Found;
//                     }
//                 }
//                 LoadSymbols = (0 < Found);
//             }
//
//             /* Move on to the next option */
//             while (*CommandLine && !isspace(*CommandLine))
//                 ++CommandLine;
//         }
//
// #if 1 // FIXME: This is a workaround HACK!!
// // Save the actual value of LoadSymbols but disable it for BootPhase 0.
//         OrigLoadSymbols = LoadSymbols;
//         LoadSymbols = FALSE;
//         return OrigLoadSymbols;
// #endif
//     }
//     else if (BootPhase == 1)
//     {
//         HANDLE Thread;
//         NTSTATUS Status;
//         KIRQL OldIrql;
//         PLIST_ENTRY ListEntry;
//
// #if 1 // FIXME: This is a workaround HACK!!
// // Now, restore the actual value of LoadSymbols.
//         LoadSymbols = OrigLoadSymbols;
// #endif
//
//         /* Do not continue loading symbols if we have less than 96MB of RAM */
//         if (MmNumberOfPhysicalPages < (96 * 1024 * 1024 / PAGE_SIZE))
//             LoadSymbols = FALSE;
//
//         /* Continue this phase only if we need to load symbols */
//         if (!LoadSymbols)
//             return LoadSymbols;
//
//         /* Launch our worker thread */
//         InitializeListHead(&SymbolsToLoad);
//         KeInitializeSpinLock(&SymbolsToLoadLock);
//         KeInitializeEvent(&SymbolsToLoadEvent, SynchronizationEvent, FALSE);
//
//         Status = PsCreateSystemThread(&Thread,
//                                       THREAD_ALL_ACCESS,
//                                       NULL, NULL, NULL,
//                                       LoadSymbolsRoutine,
//                                       NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed starting symbols loader thread: 0x%08x\n", Status);
//             LoadSymbols = FALSE;
//             return LoadSymbols;
//         }
//
//         RosSymInitKernelMode();
//
//         KeAcquireSpinLock(&PsLoadedModuleSpinLock, &OldIrql);
//
//         for (ListEntry = PsLoadedModuleList.Flink;
//              ListEntry != &PsLoadedModuleList;
//              ListEntry = ListEntry->Flink)
//         {
//             PLDR_DATA_TABLE_ENTRY LdrEntry = CONTAINING_RECORD(ListEntry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
//             KdbSymProcessSymbols(LdrEntry, TRUE);
//         }
//
//         KeReleaseSpinLock(&PsLoadedModuleSpinLock, OldIrql);
//     }
//
//     return LoadSymbols;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
