use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::CONTAINING_RECORD;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _RTL_SPLAY_LINKS,
    PRTL_SPLAY_LINKS,
    PTUNNEL,
    PUNICODE_STRING,
    LARGE_INTEGER,
    PKEY_VALUE_FULL_INFORMATION,
    LPWSTR,
    ExAcquireFastMutex,
    ExReleaseFastMutex,
    ExInitializeFastMutex,
    ExAllocatePool,
    ExFreePool,
    ExInitializePagedLookasideList,
    RtlCompareUnicodeString,
    RtlCopyUnicodeString,
    RtlDelete,
    RtlDeleteNoSplay,
    RtlRealSuccessor,
    ZwOpenKey,
    ZwQueryValueKey,
    ZwClose,
    MmIsThisAnNtAsSystem,
    _PAGED_LOOKASIDE_LIST,
    _UNICODE_STRING, HANDLE, _OBJECT_ATTRIBUTES,
};

extern "system" {
    fn RtlMoveMemory(destination: PVOID, source: PVOID, length: usize);
}
extern "system" {
    // NTAPI (stdcall on i386) per ntoskrnl.spec
    fn ExAllocateFromPagedLookasideList(lookaside: *mut _PAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToPagedLookasideList(lookaside: *mut _PAGED_LOOKASIDE_LIST, entry: PVOID);
}
extern "C" {
    fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: PUNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID, );
    fn RtlInitializeSplayLinks(links: PRTL_SPLAY_LINKS);
    fn RtlInsertAsLeftChild(parent: PRTL_SPLAY_LINKS, child: PRTL_SPLAY_LINKS);
    fn RtlInsertAsRightChild(parent: PRTL_SPLAY_LINKS, child: PRTL_SPLAY_LINKS);
    fn RtlParent(links: PRTL_SPLAY_LINKS) -> PRTL_SPLAY_LINKS;
    fn RtlIsLeftChild(links: PRTL_SPLAY_LINKS) -> BOOLEAN;
}
extern "system" {
    fn KeQuerySystemTime(current_time: *mut LARGE_INTEGER);
}

#[repr(C)]
struct TUNNEL_NODE_ENTRY {
    SplayInfo: _RTL_SPLAY_LINKS,
    TimerQueueEntry: LIST_ENTRY,
    Time: LARGE_INTEGER,
    DirectoryKey: ULONGLONG,
    Flags: ULONG,
    LongName: _UNICODE_STRING,
    ShortName: _UNICODE_STRING,
    Data: PVOID,
    DataLength: ULONG,
}

type PTUNNEL_NODE_ENTRY = *mut TUNNEL_NODE_ENTRY;

static mut TunnelMaxEntries: ULONG = 256;
static mut TunnelMaxAge: ULONG = 15;
static mut TunnelLookasideList: _PAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };

const DEFAULT_EXTRA_SIZE: ULONG = 72;
const DEFAULT_ENTRY_SIZE: ULONG = core::mem::size_of::<TUNNEL_NODE_ENTRY>() as ULONG + DEFAULT_EXTRA_SIZE;

const TUNNEL_FLAG_POOL: ULONG = 0x2;
const TUNNEL_FLAG_KEY_SHORT_NAME: ULONG = 0x1;

unsafe fn FsRtlFreeTunnelNode(
    CurEntry: PTUNNEL_NODE_ENTRY,
    PoolList: PLIST_ENTRY,
) {
    if !PoolList.is_null() {
        InsertHeadList(PoolList, core::ptr::addr_of_mut!((*CurEntry).TimerQueueEntry) as PLIST_ENTRY);
        return;
    }

    if (*CurEntry).Flags & TUNNEL_FLAG_POOL != 0 {
        ExFreePool(CurEntry as PVOID);
    } else {
        ExFreeToPagedLookasideList(core::ptr::addr_of_mut!(TunnelLookasideList), CurEntry as PVOID);
    }
}

unsafe fn FsRtlRemoveNodeFromTunnel(
    Cache: PTUNNEL,
    CurEntry: PTUNNEL_NODE_ENTRY,
    PoolList: PLIST_ENTRY,
    Rebalance: PBOOLEAN,
) {
    if !Rebalance.is_null() && *Rebalance != 0 {
        (*Cache).Cache = RtlDelete(core::ptr::addr_of_mut!((*CurEntry).SplayInfo) as PRTL_SPLAY_LINKS);
        *Rebalance = 0u8;
    } else {
        RtlDeleteNoSplay(core::ptr::addr_of_mut!((*CurEntry).SplayInfo) as PRTL_SPLAY_LINKS, core::ptr::addr_of_mut!((*Cache).Cache));
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*CurEntry).TimerQueueEntry) as PLIST_ENTRY);

    FsRtlFreeTunnelNode(CurEntry, PoolList);

    (*Cache).NumEntries -= 1;
}

unsafe fn FsRtlPruneTunnelCache(
    Cache: PTUNNEL,
    PoolList: PLIST_ENTRY,
) {
    let mut Entry: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut CurEntry: PTUNNEL_NODE_ENTRY;
    let mut CurTime: LARGE_INTEGER = core::mem::zeroed();
    let mut OldTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Rebalance: BOOLEAN = 1u8;

    KeQuerySystemTime(&mut CurTime);

    OldTime.QuadPart = CurTime.QuadPart - TunnelMaxAge as i64;

    Entry = (*Cache).TimerQueue.Flink;

    while Entry != core::ptr::addr_of_mut!((*Cache).TimerQueue) as PLIST_ENTRY {
        CurEntry = CONTAINING_RECORD!(Entry, TUNNEL_NODE_ENTRY, TimerQueueEntry) as PTUNNEL_NODE_ENTRY;

        NextEntry = (*Entry).Flink;

        if (*CurEntry).Time.QuadPart < OldTime.QuadPart || (*CurEntry).Time.QuadPart > CurTime.QuadPart {
            FsRtlRemoveNodeFromTunnel(Cache, CurEntry, PoolList, &mut Rebalance);
        }

        Entry = NextEntry;
    }

    while (*Cache).NumEntries as ULONG > TunnelMaxEntries {
        CurEntry = CONTAINING_RECORD!((*Cache).TimerQueue.Flink, TUNNEL_NODE_ENTRY, TimerQueueEntry) as PTUNNEL_NODE_ENTRY;
        FsRtlRemoveNodeFromTunnel(Cache, CurEntry, PoolList, &mut Rebalance);
    }
}

// CODE_SEG("INIT")
unsafe fn FsRtlGetTunnelParameterValue(
    ParameterName: PUNICODE_STRING,
    Value: *mut ULONG,
) {
    // L"Registry\\Machine\\System\\CurrentControlSet\\Control\\FileSystem" — UTF-16LE
    static FS_REG_PATH: [u16; 60] = [
        0x52, 0x65, 0x67, 0x69, 0x73, 0x74, 0x72, 0x79, // Registry
        0x5C, 0x4D, 0x61, 0x63, 0x68, 0x69, 0x6E, 0x65, // \Machine
        0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x5C, // \System\backslash
        0x43, 0x75, 0x72, 0x72, 0x65, 0x6E, 0x74, 0x43, // CurrentC
        0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C, 0x53, 0x65, // ontrolSe
        0x74, 0x5C, 0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, // t\Contro
        0x6C, 0x5C, 0x46, 0x69, 0x6C, 0x65, 0x53, 0x79, // l\FileSy
        0x73, 0x74, 0x65, 0x6D,                            // stem
    ];
    let mut Root: _UNICODE_STRING = _UNICODE_STRING {
        Length: 118,        // 59 chars * 2
        MaximumLength: 120, // 60 u16s * 2
        Buffer: FS_REG_PATH.as_ptr() as LPWSTR,
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Length: ULONG = 0;
    let mut Info: PKEY_VALUE_FULL_INFORMATION;

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut Root,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = ZwOpenKey(&mut hKey, KEY_READ, &mut ObjectAttributes);

    if Status < 0 {
        return;
    }

    Status = ZwQueryValueKey(hKey, ParameterName, ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation as _, core::ptr::null_mut(), 0, &mut Length);

    if Status != STATUS_BUFFER_TOO_SMALL {
        ZwClose(hKey);
        return;
    }

    Info = ExAllocatePool(PagedPool, Length as _) as PKEY_VALUE_FULL_INFORMATION;

    if Info.is_null() {
        ZwClose(hKey);
        return;
    }

    Status = ZwQueryValueKey(hKey, ParameterName, ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation as _, Info as PVOID, Length, &mut Length);

    if Status >= 0 {
        if (*Info).DataLength != 0 {
            *Value = *((Info as ULONG_PTR + (*Info).DataOffset as ULONG_PTR) as *const ULONG);
        }
    }

    ExFreePool(Info as PVOID);

    ZwClose(hKey);
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeTunnels() {
    let mut TunnelEntries: ULONG;
    // L"MaximumTunnelEntryAgeInSeconds" — UTF-16LE
    static MAX_AGE_NAME: [u16; 30] = [
        0x4D, 0x61, 0x78, 0x69, 0x6D, 0x75, 0x6D, 0x54, // MaximumT
        0x75, 0x6E, 0x6E, 0x65, 0x6C, 0x45, 0x6E, 0x74, // unnelEnt
        0x72, 0x79, 0x41, 0x67, 0x65, 0x49, 0x6E, 0x53, // ryAgeInS
        0x65, 0x63, 0x6F, 0x6E, 0x64, 0x73,              // econds
    ];
    let mut MaximumTunnelEntryAgeInSeconds: _UNICODE_STRING = _UNICODE_STRING {
        Length: 58,        // 29 chars * 2
        MaximumLength: 60, // 30 u16s * 2
        Buffer: MAX_AGE_NAME.as_ptr() as LPWSTR,
    };
    // L"MaximumTunnelEntries" — UTF-16LE
    static MAX_ENTRIES_NAME: [u16; 21] = [
        0x4D, 0x61, 0x78, 0x69, 0x6D, 0x75, 0x6D, 0x54, // MaximumT
        0x75, 0x6E, 0x6E, 0x65, 0x6C, 0x45, 0x6E, 0x74, // unnelEnt
        0x72, 0x69, 0x65, 0x73, 0x00,                      // ries\0
    ];
    let mut MaximumTunnelEntries: _UNICODE_STRING = _UNICODE_STRING {
        Length: 40,        // 20 chars * 2
        MaximumLength: 42, // 21 u16s * 2
        Buffer: MAX_ENTRIES_NAME.as_ptr() as LPWSTR,
    };

    if MmIsThisAnNtAsSystem() != 0 {
        TunnelMaxEntries = 1024;
    }

    FsRtlGetTunnelParameterValue(&mut MaximumTunnelEntries, &raw mut TunnelMaxEntries);

    FsRtlGetTunnelParameterValue(&mut MaximumTunnelEntryAgeInSeconds, &raw mut TunnelMaxAge);

    if TunnelMaxAge == 0 {
        TunnelMaxEntries = 0;
    }

    TunnelEntries = TunnelMaxEntries;

    TunnelMaxAge *= 10000000;

    if TunnelMaxEntries <= 65535 {
        TunnelEntries = TunnelMaxEntries / 16;
    }

    if TunnelEntries == 0 && TunnelMaxEntries != 0 {
        TunnelEntries = TunnelMaxEntries + 1;
    }

    if TunnelEntries > 0xFFFF {
        TunnelEntries = 256;
    }

    ExInitializePagedLookasideList(
        core::ptr::addr_of_mut!(TunnelLookasideList),
        None,
        None,
        0,
        DEFAULT_ENTRY_SIZE as _,
        0x4C6E7554,
        TunnelEntries as u16,
    );
}

unsafe fn FsRtlCompareNodeAndKey(
    CurEntry: PTUNNEL_NODE_ENTRY,
    DirectoryKey: ULONGLONG,
    KeyString: PUNICODE_STRING,
) -> LONG {
    let String: PUNICODE_STRING;
    let Ret: LONG;

    if DirectoryKey > (*CurEntry).DirectoryKey {
        Ret = 1;
    } else if DirectoryKey < (*CurEntry).DirectoryKey {
        Ret = -1;
    } else {
        if (*CurEntry).Flags & TUNNEL_FLAG_KEY_SHORT_NAME != 0 {
            String = core::ptr::addr_of_mut!((*CurEntry).ShortName);
        } else {
            String = core::ptr::addr_of_mut!((*CurEntry).LongName);
        }

        Ret = RtlCompareUnicodeString(KeyString, String, 1u8);
    }

    Ret
}

unsafe fn FsRtlEmptyFreePoolList(PoolList: PLIST_ENTRY) {
    let mut CurEntry: PLIST_ENTRY;
    let mut CurNode: PTUNNEL_NODE_ENTRY;

    while !IsListEmpty(PoolList) {
        CurEntry = RemoveHeadList(PoolList);
        CurNode = CONTAINING_RECORD!(CurEntry, TUNNEL_NODE_ENTRY, TimerQueueEntry) as PTUNNEL_NODE_ENTRY;
        FsRtlFreeTunnelNode(CurNode, core::ptr::null_mut());
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAddToTunnelCache(
    Cache: PTUNNEL,
    DirectoryKey: ULONGLONG,
    ShortName: PUNICODE_STRING,
    LongName: PUNICODE_STRING,
    KeyByShortName: BOOLEAN,
    DataLength: ULONG,
    Data: PVOID,
) {
    let ShortName = opaque_ptr(ShortName);
    let LongName = opaque_ptr(LongName);
    let Data = opaque_ptr(Data);
    let mut NodeEntry: PTUNNEL_NODE_ENTRY = core::ptr::null_mut();
    let mut CurEntry: PRTL_SPLAY_LINKS;
    let mut LastEntry: PRTL_SPLAY_LINKS;
    let mut Length: ULONG;
    let mut Result: LONG = 0;
    let mut AllocatedFromPool: BOOLEAN = 0u8;
    let KeyString: PUNICODE_STRING;
    let mut PoolList: LIST_ENTRY = core::mem::zeroed();

    if TunnelMaxEntries == 0 {
        return;
    }

    InitializeListHead(&mut PoolList);

    Length = core::mem::size_of::<TUNNEL_NODE_ENTRY>() as ULONG;

    Length += DataLength;

    if !ShortName.is_null() {
        Length += (*ShortName).Length as ULONG;
    }

    if !LongName.is_null() {
        Length += (*LongName).Length as ULONG;
    }

    if Length <= DEFAULT_ENTRY_SIZE {
        NodeEntry = ExAllocateFromPagedLookasideList(core::ptr::addr_of_mut!(TunnelLookasideList)) as PTUNNEL_NODE_ENTRY;
    }

    if NodeEntry.is_null() {
        NodeEntry = ExAllocatePool(PagedPool | 0x100, Length as _) as PTUNNEL_NODE_ENTRY;
        if NodeEntry.is_null() {
            return;
        }

        AllocatedFromPool = 1u8;
    }

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*Cache).Mutex));

    CurEntry = (*Cache).Cache as PRTL_SPLAY_LINKS;

    KeyString = if KeyByShortName != 0 { ShortName } else { LongName };

    LastEntry = core::ptr::null_mut();

    while !CurEntry.is_null() {
        Result = FsRtlCompareNodeAndKey(CurEntry as PTUNNEL_NODE_ENTRY, DirectoryKey, KeyString);

        LastEntry = CurEntry;

        if Result > 0 {
            CurEntry = (*CurEntry).LeftChild;
        } else {
            if Result == 0 {
                break;
            }

            CurEntry = (*CurEntry).RightChild;
        }
    }

    RtlInitializeSplayLinks(core::ptr::addr_of_mut!((*NodeEntry).SplayInfo) as PRTL_SPLAY_LINKS);

    if !CurEntry.is_null() {
        if !(*CurEntry).LeftChild.is_null() {
            RtlInsertAsLeftChild(core::ptr::addr_of_mut!((*NodeEntry).SplayInfo) as PRTL_SPLAY_LINKS, (*CurEntry).LeftChild);
        }

        if !(*CurEntry).RightChild.is_null() {
            RtlInsertAsRightChild(core::ptr::addr_of_mut!((*NodeEntry).SplayInfo) as PRTL_SPLAY_LINKS, (*CurEntry).RightChild);
        }

        if (*CurEntry).Parent == CurEntry {
            (*Cache).Cache = NodeEntry as *mut _RTL_SPLAY_LINKS;
        } else {
            if RtlIsLeftChild(CurEntry) != 0 {
                RtlInsertAsLeftChild(RtlParent(CurEntry), core::ptr::addr_of_mut!((*NodeEntry).SplayInfo) as PRTL_SPLAY_LINKS);
            } else {
                RtlInsertAsRightChild(RtlParent(CurEntry), core::ptr::addr_of_mut!((*NodeEntry).SplayInfo) as PRTL_SPLAY_LINKS);
            }
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*(CurEntry as PTUNNEL_NODE_ENTRY)).TimerQueueEntry) as PLIST_ENTRY);

        FsRtlFreeTunnelNode(CurEntry as PTUNNEL_NODE_ENTRY, &mut PoolList);

        (*Cache).NumEntries -= 1;
    } else {
        if LastEntry.is_null() {
            (*Cache).Cache = NodeEntry as *mut _RTL_SPLAY_LINKS;
        } else {
            if Result > 0 {
                RtlInsertAsLeftChild(LastEntry, core::ptr::addr_of_mut!((*NodeEntry).SplayInfo) as PRTL_SPLAY_LINKS);
            } else {
                RtlInsertAsRightChild(LastEntry, core::ptr::addr_of_mut!((*NodeEntry).SplayInfo) as PRTL_SPLAY_LINKS);
            }
        }
    }

    KeQuerySystemTime(&mut (*NodeEntry).Time);

    (*NodeEntry).DirectoryKey = DirectoryKey;
    (*NodeEntry).Flags = if AllocatedFromPool != 0 { TUNNEL_FLAG_POOL } else { 0x0 };
    (*NodeEntry).Flags |= if KeyByShortName != 0 { TUNNEL_FLAG_KEY_SHORT_NAME } else { 0x0 };

    if !ShortName.is_null() {
        (*NodeEntry).ShortName.Length = (*ShortName).Length;
        (*NodeEntry).ShortName.MaximumLength = (*ShortName).Length;
        (*NodeEntry).ShortName.Buffer = ((NodeEntry as ULONG_PTR) + core::mem::size_of::<TUNNEL_NODE_ENTRY>()) as LPWSTR;

        RtlMoveMemory((*NodeEntry).ShortName.Buffer as PVOID, (*ShortName).Buffer as PVOID, (*ShortName).Length as usize);
    } else {
        (*NodeEntry).ShortName.Length = 0;
        (*NodeEntry).ShortName.MaximumLength = 0;
        (*NodeEntry).ShortName.Buffer = core::ptr::null_mut();
    }

    if !LongName.is_null() {
        (*NodeEntry).LongName.Length = (*LongName).Length;
        (*NodeEntry).LongName.MaximumLength = (*LongName).Length;
        (*NodeEntry).LongName.Buffer = ((NodeEntry as ULONG_PTR) + core::mem::size_of::<TUNNEL_NODE_ENTRY>() + (*NodeEntry).ShortName.Length as usize) as LPWSTR;

        RtlMoveMemory((*NodeEntry).LongName.Buffer as PVOID, (*LongName).Buffer as PVOID, (*LongName).Length as usize);
    } else {
        (*NodeEntry).LongName.Length = 0;
        (*NodeEntry).LongName.MaximumLength = 0;
        (*NodeEntry).LongName.Buffer = core::ptr::null_mut();
    }

    (*NodeEntry).DataLength = DataLength;
    (*NodeEntry).Data = ((NodeEntry as ULONG_PTR) + core::mem::size_of::<TUNNEL_NODE_ENTRY>() + (*NodeEntry).ShortName.Length as usize + (*NodeEntry).LongName.Length as usize) as PVOID;
    RtlMoveMemory((*NodeEntry).Data, Data, DataLength as usize);

    (*Cache).NumEntries += 1;

    InsertTailList(core::ptr::addr_of_mut!((*Cache).TimerQueue) as PLIST_ENTRY, core::ptr::addr_of_mut!((*NodeEntry).TimerQueueEntry) as PLIST_ENTRY);

    FsRtlPruneTunnelCache(Cache, &mut PoolList);

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*Cache).Mutex));

    FsRtlEmptyFreePoolList(&mut PoolList);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDeleteKeyFromTunnelCache(
    Cache: PTUNNEL,
    DirectoryKey: ULONGLONG,
) {
    let mut Rebalance: BOOLEAN = 1u8;
    let mut PoolList: LIST_ENTRY = core::mem::zeroed();
    let mut CurNode: PTUNNEL_NODE_ENTRY;
    let mut CurEntry: PRTL_SPLAY_LINKS;
    let mut LastEntry: PRTL_SPLAY_LINKS = core::ptr::null_mut();
    let mut Successors: PRTL_SPLAY_LINKS;

    if TunnelMaxEntries == 0 {
        return;
    }

    InitializeListHead(&mut PoolList);

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*Cache).Mutex));

    CurEntry = (*Cache).Cache as PRTL_SPLAY_LINKS;
    while !CurEntry.is_null() {
        CurNode = CONTAINING_RECORD!(CurEntry, TUNNEL_NODE_ENTRY, SplayInfo) as PTUNNEL_NODE_ENTRY;

        if (*CurNode).DirectoryKey > DirectoryKey {
            CurEntry = (*CurEntry).LeftChild;
        } else if (*CurNode).DirectoryKey < DirectoryKey {
            if !LastEntry.is_null() {
                break;
            }

            CurEntry = (*CurEntry).RightChild;
        } else {
            LastEntry = CurEntry;
            CurEntry = (*CurEntry).LeftChild;
        }
    }

    if LastEntry.is_null() {
        ExReleaseFastMutex(core::ptr::addr_of_mut!((*Cache).Mutex));
        return;
    }

    loop {
        CurNode = CONTAINING_RECORD!(LastEntry, TUNNEL_NODE_ENTRY, SplayInfo) as PTUNNEL_NODE_ENTRY;

        Successors = RtlRealSuccessor(LastEntry);
        if (*CurNode).DirectoryKey != DirectoryKey {
            break;
        }

        FsRtlRemoveNodeFromTunnel(Cache, CurNode, &mut PoolList, &mut Rebalance);
        LastEntry = Successors;

        if LastEntry.is_null() {
            break;
        }
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*Cache).Mutex));

    FsRtlEmptyFreePoolList(&mut PoolList);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDeleteTunnelCache(Cache: PTUNNEL) {
    let mut Entry: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut CurEntry: PTUNNEL_NODE_ENTRY;

    if TunnelMaxEntries == 0 {
        return;
    }

    Entry = (*Cache).TimerQueue.Flink;

    while Entry != core::ptr::addr_of_mut!((*Cache).TimerQueue) as PLIST_ENTRY {
        CurEntry = CONTAINING_RECORD!(Entry, TUNNEL_NODE_ENTRY, TimerQueueEntry) as PTUNNEL_NODE_ENTRY;

        NextEntry = (*Entry).Flink;

        RemoveEntryList(core::ptr::addr_of_mut!((*CurEntry).TimerQueueEntry) as PLIST_ENTRY);

        FsRtlFreeTunnelNode(CurEntry, core::ptr::null_mut());

        Entry = NextEntry;
    }

    (*Cache).Cache = core::ptr::null_mut();
    (*Cache).NumEntries = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Cache).TimerQueue));
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlFindInTunnelCache(
    Cache: PTUNNEL,
    DirectoryKey: ULONGLONG,
    Name: PUNICODE_STRING,
    ShortName: PUNICODE_STRING,
    LongName: PUNICODE_STRING,
    DataLength: *mut ULONG,
    Data: PVOID,
) -> BOOLEAN {
    let Name = opaque_ptr(Name);
    let ShortName = opaque_ptr(ShortName);
    let LongName = opaque_ptr(LongName);
    let DataLength = opaque_ptr(DataLength);
    let Data = opaque_ptr(Data);
    let mut Ret: BOOLEAN = 0u8;
    let mut CurEntry: PTUNNEL_NODE_ENTRY;
    let mut PoolList: LIST_ENTRY = core::mem::zeroed();
    let mut Result: LONG;

    if TunnelMaxEntries == 0 {
        return 0u8;
    }

    InitializeListHead(&mut PoolList);

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*Cache).Mutex));

    FsRtlPruneTunnelCache(Cache, &mut PoolList);

    CurEntry = (*Cache).Cache as PTUNNEL_NODE_ENTRY;

    while !CurEntry.is_null() {
        Result = FsRtlCompareNodeAndKey(CurEntry, DirectoryKey, Name);

        if Result > 0 {
            CurEntry = (*CurEntry).SplayInfo.LeftChild as PTUNNEL_NODE_ENTRY;
        } else {
            if Result == 0 {
                break;
            }

            CurEntry = (*CurEntry).SplayInfo.RightChild as PTUNNEL_NODE_ENTRY;
        }
    }

    if !CurEntry.is_null() {
        RtlCopyUnicodeString(ShortName, core::ptr::addr_of_mut!((*CurEntry).ShortName));

        if (*LongName).MaximumLength < (*CurEntry).LongName.Length {
            (*LongName).Buffer = ExAllocatePool(PagedPool, (*CurEntry).LongName.Length as _) as LPWSTR;
            if !(*LongName).Buffer.is_null() {
                (*LongName).Length = (*CurEntry).LongName.Length;
                (*LongName).MaximumLength = (*CurEntry).LongName.MaximumLength;
                RtlMoveMemory((*LongName).Buffer as PVOID, (*CurEntry).LongName.Buffer as PVOID, (*CurEntry).LongName.Length as usize);
            }
        } else {
            RtlCopyUnicodeString(LongName, core::ptr::addr_of_mut!((*CurEntry).LongName));
        }

        RtlMoveMemory(Data, (*CurEntry).Data, (*CurEntry).DataLength as usize);

        *DataLength = (*CurEntry).DataLength;

        Ret = 1u8;
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*Cache).Mutex));

    FsRtlEmptyFreePoolList(&mut PoolList);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeTunnelCache(Cache: PTUNNEL) {
    ExInitializeFastMutex(core::ptr::addr_of_mut!((*Cache).Mutex));

    (*Cache).Cache = core::ptr::null_mut();

    InitializeListHead(core::ptr::addr_of_mut!((*Cache).TimerQueue));

    (*Cache).NumEntries = 0;
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/tunnel.c
//  * PURPOSE:         Provides the Tunnel Cache implementation for file system drivers.
//  * PROGRAMMERS:     Johannes Anderwald (johannes.anderwald@reactos.org)
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// typedef struct {
//     RTL_SPLAY_LINKS SplayInfo;
//     LIST_ENTRY TimerQueueEntry;
//     LARGE_INTEGER Time;
//     ULONGLONG DirectoryKey;
//     ULONG Flags;
//     _UNICODE_STRING LongName;
//     _UNICODE_STRING ShortName;
//     PVOID Data;
//     ULONG DataLength;
// } TUNNEL_NODE_ENTRY, *PTUNNEL_NODE_ENTRY;
//
// ULONG TunnelMaxEntries = 256;
// ULONG TunnelMaxAge = 15;
// _PAGED_LOOKASIDE_LIST TunnelLookasideList;
//
// #define DEFAULT_EXTRA_SIZE (72)
// #define DEFAULT_ENTRY_SIZE (sizeof(TUNNEL_NODE_ENTRY) + DEFAULT_EXTRA_SIZE)
//
// #define TUNNEL_FLAG_POOL 0x2
// #define TUNNEL_FLAG_KEY_SHORT_NAME 0x1
//
// VOID
// FsRtlFreeTunnelNode(
//     IN PTUNNEL_NODE_ENTRY CurEntry,
//     IN PLIST_ENTRY PoolList OPTIONAL)
// {
//     if (PoolList)
//     {
//         /* divert the linked list entry, it's not required anymore, but we need it */
//         InsertHeadList(PoolList, &CurEntry->TimerQueueEntry);
//         return;
//     }
//
//     if (CurEntry->Flags & TUNNEL_FLAG_POOL)
//         ExFreePool(CurEntry);
//     else
//         ExFreeToPagedLookasideList(&TunnelLookasideList, CurEntry);
// }
//
// VOID
// FsRtlRemoveNodeFromTunnel(
//     IN PTUNNEL Cache,
//     IN PTUNNEL_NODE_ENTRY CurEntry,
//     IN PLIST_ENTRY PoolList,
//     OUT PBOOLEAN Rebalance)
// {
//     /* delete entry and rebalance if required */
//     if (Rebalance && *Rebalance)
//     {
//         Cache->Cache = RtlDelete(&CurEntry->SplayInfo);
//         /* reset */
//         *Rebalance = FALSE;
//     }
//     else
//     {
//         RtlDeleteNoSplay(&CurEntry->SplayInfo, &Cache->Cache);
//     }
//
//     /* remove entry */
//     RemoveEntryList(&CurEntry->TimerQueueEntry);
//
//     /* free node entry */
//     FsRtlFreeTunnelNode(CurEntry, PoolList);
//
//     /* decrement node count */
//     Cache->NumEntries--;
// }
//
// VOID
// FsRtlPruneTunnelCache(
//     IN PTUNNEL Cache,
//     IN PLIST_ENTRY PoolList)
// {
//     PLIST_ENTRY Entry, NextEntry;
//     PTUNNEL_NODE_ENTRY CurEntry;
//     LARGE_INTEGER CurTime, OldTime;
//     BOOLEAN Rebalance = TRUE;
//     PAGED_CODE();
//
//     /* query time */
//     KeQuerySystemTime(&CurTime);
//
//     /* subtract maximum node age */
//     OldTime.QuadPart = CurTime.QuadPart - TunnelMaxAge;
//
//     /* free all entries */
//     Entry = Cache->TimerQueue.Flink;
//
//     while(Entry != &Cache->TimerQueue)
//     {
//         /* get node entry */
//         CurEntry = CONTAINING_RECORD(Entry, TUNNEL_NODE_ENTRY, TimerQueueEntry);
//
//         /* get next entry */
//          NextEntry = Entry->Flink;
//
//         /* prune if expired OR if in advance in time */
//         if (CurEntry->Time.QuadPart < OldTime.QuadPart ||
//             CurEntry->Time.QuadPart > CurTime.QuadPart)
//         {
//             FsRtlRemoveNodeFromTunnel(Cache, CurEntry, PoolList, &Rebalance);
//         }
//
//         /* move to next entry */
//         Entry = NextEntry;
//     }
//
//     /* If we have too many entries */
//     while (Cache->NumEntries > TunnelMaxEntries)
//     {
//         ASSERT(!IsListEmpty(&Cache->TimerQueue));
//         CurEntry = CONTAINING_RECORD(Cache->TimerQueue.Flink, TUNNEL_NODE_ENTRY, TimerQueueEntry);
//         FsRtlRemoveNodeFromTunnel(Cache, CurEntry, PoolList, &Rebalance);
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// FsRtlGetTunnelParameterValue(
//     IN PUNICODE_STRING ParameterName,
//     OUT PULONG Value)
// {
//     _UNICODE_STRING Root = RTL_CONSTANT_STRING(L"Registry\\Machine\\System\\CurrentControlSet\\Control\\FileSystem");
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hKey;
//     NTSTATUS Status;
//     ULONG Length;
//     PKEY_VALUE_FULL_INFORMATION Info;
//
//     /* initialize object attributes */
//     InitializeObjectAttributes(&ObjectAttributes, &Root, OBJ_CASE_INSENSITIVE, NULL, NULL);
//
//     /* open registry key */
//     Status = ZwOpenKey(&hKey, KEY_READ, &ObjectAttributes);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to open key */
//         return;
//     }
//
//     /* query value size */
//     Status = ZwQueryValueKey(hKey, ParameterName, KeyValueFullInformation, NULL, 0, &Length);
//
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         /* failed to query size */
//         ZwClose(hKey);
//         return;
//     }
//
//     /* allocate buffer */
//     Info = ExAllocatePool(PagedPool, Length);
//
//     if (!Info)
//     {
//          /* out of memory */
//         ZwClose(hKey);
//         return;
//     }
//
//     /* query value */
//     Status = ZwQueryValueKey(hKey, ParameterName, KeyValueFullInformation, NULL, 0, &Length);
//
//     if (NT_SUCCESS(Status))
//     {
//         if (Info->DataLength)
//         {
//             /* store result */
//             *Value = (ULONG)((ULONG_PTR)Info + Info->DataOffset);
//         }
//     }
//
//     /* free buffer */
//     ExFreePool(Info);
//
//     /* close key */
//     ZwClose(hKey);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// FsRtlInitializeTunnels(VOID)
// {
//     ULONG TunnelEntries;
//     _UNICODE_STRING MaximumTunnelEntryAgeInSeconds = RTL_CONSTANT_STRING(L"MaximumTunnelEntryAgeInSeconds");
//     _UNICODE_STRING MaximumTunnelEntries = RTL_CONSTANT_STRING( L"MaximumTunnelEntries");
//
//     /* check for nt */
//     if (MmIsThisAnNtAsSystem())
//     {
//         /* default */
//         TunnelMaxEntries = 1024;
//     }
//
//     /* check for custom override of max entries*/
//     FsRtlGetTunnelParameterValue(&MaximumTunnelEntries, &TunnelMaxEntries);
//
//     /* check for custom override of age*/
//     FsRtlGetTunnelParameterValue(&MaximumTunnelEntryAgeInSeconds, &TunnelMaxAge);
//
//     if (!TunnelMaxAge)
//     {
//         /* no age means no entries */
//         TunnelMaxEntries = 0;
//     }
//
//     /* get max entries */
//     TunnelEntries = TunnelMaxEntries;
//
//     /* convert to ticks */
//     TunnelMaxAge *= 10000000;
//
//     if(TunnelMaxEntries <= 65535)
//     {
//         /* use max 256 entries */
//         TunnelEntries = TunnelMaxEntries / 16;
//     }
//
//     if(!TunnelEntries && TunnelMaxEntries )
//     {
//         /* max tunnel entries was too small */
//         TunnelEntries = TunnelMaxEntries + 1;
//     }
//
//     if (TunnelEntries > 0xFFFF)
//     {
//         /* max entries is 256 */
//         TunnelEntries = 256;
//     }
//
//     /* initialize look aside list */
//     ExInitializePagedLookasideList(&TunnelLookasideList, NULL, NULL, 0, DEFAULT_ENTRY_SIZE, 'TunL', TunnelEntries);
// }
//
// LONG
// FsRtlCompareNodeAndKey(
//     IN PTUNNEL_NODE_ENTRY CurEntry,
//     IN ULONGLONG DirectoryKey,
//     IN PUNICODE_STRING KeyString)
// {
//     PUNICODE_STRING String;
//     LONG Ret;
//
//     if (DirectoryKey > CurEntry->DirectoryKey)
//     {
//         Ret = 1;
//     }
//     else if (DirectoryKey < CurEntry->DirectoryKey)
//     {
//         Ret = -1;
//     }
//     else
//     {
//         if (CurEntry->Flags & TUNNEL_FLAG_KEY_SHORT_NAME)
//         {
//             /* use short name as key */
//             String = &CurEntry->ShortName;
//         }
//         else
//         {
//             /* use long name as key */
//             String = &CurEntry->LongName;
//         }
//
//         Ret = RtlCompareUnicodeString(KeyString, String, TRUE);
//     }
//
//     return Ret;
// }
//
// VOID
// FsRtlEmptyFreePoolList(
//     IN PLIST_ENTRY PoolList)
// {
//     PLIST_ENTRY CurEntry;
//     PTUNNEL_NODE_ENTRY CurNode;
//
//     /* loop over all the entry */
//     while (!IsListEmpty(PoolList))
//     {
//         /* and free them, one by one */
//         CurEntry = RemoveHeadList(PoolList);
//         CurNode = CONTAINING_RECORD(CurEntry, TUNNEL_NODE_ENTRY, TimerQueueEntry);
//         FsRtlFreeTunnelNode(CurNode, 0);
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlAddToTunnelCache
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Cache
//  *        FILLME
//  *
//  * @param DirectoryKey
//  *        FILLME
//  *
//  * @param ShortName
//  *        FILLME
//  *
//  * @param LongName
//  *        FILLME
//  *
//  * @param KeyByShortName
//  *        FILLME
//  *
//  * @param DataLength
//  *        FILLME
//  *
//  * @param Data
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlAddToTunnelCache(IN PTUNNEL Cache,
//                       IN ULONGLONG DirectoryKey,
//                       IN PUNICODE_STRING ShortName,
//                       IN PUNICODE_STRING LongName,
//                       IN BOOLEAN KeyByShortName,
//                       IN ULONG DataLength,
//                       IN PVOID Data)
// {
//     PTUNNEL_NODE_ENTRY NodeEntry = NULL;
//     PRTL_SPLAY_LINKS CurEntry, LastEntry;
//     ULONG Length;
//     LONG Result = 0;
//     BOOLEAN AllocatedFromPool = FALSE;
//     PUNICODE_STRING KeyString;
//     LIST_ENTRY PoolList;
//
//     PAGED_CODE();
//
//     /* check if tunnel cache is enabled */
//     if (!TunnelMaxEntries)
//     {
//         /* entries are disabled */
//         return;
//     }
//
//     /* initialize free pool list */
//     InitializeListHead(&PoolList);
//
//     /* calculate node length */
//     Length = sizeof(TUNNEL_NODE_ENTRY);
//
//     /* add data size */
//     Length += DataLength;
//
//     if (ShortName)
//     {
//         /* add short name length */
//         Length += ShortName->Length;
//     }
//
//     if (LongName)
//     {
//         /* add short name length */
//         Length += LongName->Length;
//     }
//
//     if (Length <= DEFAULT_ENTRY_SIZE)
//     {
//         /* get standard entry */
//         NodeEntry = ExAllocateFromPagedLookasideList(&TunnelLookasideList);
//     }
//
//     if (NodeEntry == NULL)
//     {
//         /* bigger than default entry or allocation failed */
//         NodeEntry = ExAllocatePool(PagedPool | POOL_COLD_ALLOCATION, Length);
//         /* check for success */
//         if (NodeEntry == NULL)
//         {
//              /* out of memory */
//              return;
//         }
//
//         AllocatedFromPool = TRUE;
//     }
//
//     /* acquire lock */
//     ExAcquireFastMutex(&Cache->Mutex);
//
//     /* now search cache for existing entries */
//     CurEntry = Cache->Cache;
//
//     /* check which key should be used for search */
//     KeyString = (KeyByShortName ? ShortName : LongName);
//
//     /* initialize last entry */
//     LastEntry = NULL;
//
//     while(CurEntry)
//     {
//         /* compare current node */
//         Result = FsRtlCompareNodeAndKey((PTUNNEL_NODE_ENTRY)CurEntry, DirectoryKey, KeyString);
//
//         /* backup last entry */
//         LastEntry = CurEntry;
//
//         if (Result > 0)
//         {
//             /* current directory key is bigger */
//             CurEntry = CurEntry->LeftChild;
//         }
//         else
//         {
//             if (Result == 0)
//             {
//                 /* found equal entry */
//                 break;
//             }
//
//             /* current directory key is smaller */
//             CurEntry = CurEntry->RightChild;
//         }
//     }
//
//     /* initialize node entry */
//     RtlInitializeSplayLinks(&NodeEntry->SplayInfo);
//
//     if (CurEntry != NULL)
//     {
//          /* found existing item */
//          if (CurEntry->LeftChild)
//          {
//              /* update parent */
//              RtlInsertAsLeftChild(NodeEntry, CurEntry->LeftChild);
//          }
//
//          if (CurEntry->RightChild)
//          {
//              /* update parent */
//              RtlInsertAsRightChild(NodeEntry, CurEntry->RightChild);
//          }
//
//          if (CurEntry->Parent == CurEntry)
//          {
//               /* cur entry was root */
//               Cache->Cache = (struct _RTL_SPLAY_LINKS*)NodeEntry;
//          }
//          else
//          {
//               /* update parent node */
//               if (RtlIsLeftChild(CurEntry))
//               {
//                   RtlInsertAsLeftChild(RtlParent(CurEntry), NodeEntry);
//               }
//               else
//               {
//                   RtlInsertAsRightChild(RtlParent(CurEntry), NodeEntry);
//               }
//          }
//
//          /* remove entry */
//          RemoveEntryList(&((PTUNNEL_NODE_ENTRY)CurEntry)->TimerQueueEntry);
//
//          /* free node entry */
//          FsRtlFreeTunnelNode((PTUNNEL_NODE_ENTRY)CurEntry, &PoolList);
//
//          /* decrement node count */
//          Cache->NumEntries--;
//     }
//     else
//     {
//         if (LastEntry == NULL)
//         {
//             /* first entry in tunnel cache */
//             Cache->Cache = (struct _RTL_SPLAY_LINKS*)NodeEntry;
//         }
//         else
//         {
//             if (Result > 0)
//             {
//                 /* new left node */
//                 RtlInsertAsLeftChild(LastEntry, NodeEntry);
//             }
//             else
//             {
//                 /* new right node */
//                 RtlInsertAsRightChild(LastEntry, NodeEntry);
//             }
//         }
//     }
//
//     /* initialize entry */
//     KeQuerySystemTime(&NodeEntry->Time);
//
//     NodeEntry->DirectoryKey = DirectoryKey;
//     NodeEntry->Flags = (AllocatedFromPool ? TUNNEL_FLAG_POOL : 0x0);
//     NodeEntry->Flags |= (KeyByShortName ? TUNNEL_FLAG_KEY_SHORT_NAME : 0x0);
//
//     if (ShortName)
//     {
//         /* copy short name */
//         NodeEntry->ShortName.Length = ShortName->Length;
//         NodeEntry->ShortName.MaximumLength = ShortName->Length;
//         NodeEntry->ShortName.Buffer = (LPWSTR)((ULONG_PTR)NodeEntry + sizeof(TUNNEL_NODE_ENTRY));
//
//         RtlMoveMemory(NodeEntry->ShortName.Buffer, ShortName->Buffer, ShortName->Length);
//     }
//     else
//     {
//         NodeEntry->ShortName.Length = NodeEntry->ShortName.MaximumLength = 0;
//         NodeEntry->ShortName.Buffer = NULL;
//     }
//
//     if (LongName)
//     {
//         /* copy long name */
//         NodeEntry->LongName.Length = LongName->Length;
//         NodeEntry->LongName.MaximumLength = LongName->Length;
//         NodeEntry->LongName.Buffer = (LPWSTR)((ULONG_PTR)NodeEntry + sizeof(TUNNEL_NODE_ENTRY) + NodeEntry->ShortName.Length);
//
//         RtlMoveMemory(NodeEntry->LongName.Buffer, LongName->Buffer, LongName->Length);
//     }
//     else
//     {
//         NodeEntry->LongName.Length = NodeEntry->LongName.MaximumLength = 0;
//         NodeEntry->LongName.Buffer = NULL;
//     }
//
//      NodeEntry->DataLength = DataLength;
//      NodeEntry->Data = (PVOID)((ULONG_PTR)NodeEntry + sizeof(TUNNEL_NODE_ENTRY) + NodeEntry->ShortName.Length + NodeEntry->LongName.Length);
//      RtlMoveMemory(NodeEntry->Data, Data, DataLength);
//
//      /* increment node count */
//      Cache->NumEntries++;
//
//      /* insert into list */
//      InsertTailList(&Cache->TimerQueue, &NodeEntry->TimerQueueEntry);
//
//      /* prune cache */
//      FsRtlPruneTunnelCache(Cache, &PoolList);
//
//      /* release lock */
//      ExReleaseFastMutex(&Cache->Mutex);
//
//      /* free pool list */
//      FsRtlEmptyFreePoolList(&PoolList);
// }
//
// /*++
//  * @name FsRtlDeleteKeyFromTunnelCache
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Cache
//  *        FILLME
//  *
//  * @param DirectoryKey
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlDeleteKeyFromTunnelCache(IN PTUNNEL Cache,
//                               IN ULONGLONG DirectoryKey)
// {
//     BOOLEAN Rebalance = TRUE;
//     LIST_ENTRY PoolList;
//     PTUNNEL_NODE_ENTRY CurNode;
//     PRTL_SPLAY_LINKS CurEntry, LastEntry = NULL, Successors;
//
//     PAGED_CODE();
//
//     /* check if tunnel cache is enabled */
//     if (!TunnelMaxEntries)
//     {
//         /* entries are disabled */
//         return;
//     }
//
//     /* initialize free pool list */
//     InitializeListHead(&PoolList);
//
//     /* acquire lock */
//     ExAcquireFastMutex(&Cache->Mutex);
//
//     /* Look for the entry */
//     CurEntry = Cache->Cache;
//     while (CurEntry)
//     {
//         CurNode = CONTAINING_RECORD(CurEntry, TUNNEL_NODE_ENTRY, SplayInfo);
//
//         if (CurNode->DirectoryKey > DirectoryKey)
//         {
//             /* current directory key is bigger */
//             CurEntry = CurEntry->LeftChild;
//         }
//         else if (CurNode->DirectoryKey < DirectoryKey)
//         {
//             /* if we have already found one suitable, break */
//             if (LastEntry != NULL)
//             {
//                 break;
//             }
//
//             /* current directory key is smaller */
//             CurEntry = CurEntry->RightChild;
//         }
//         else
//         {
//             /* save and look for another */
//             LastEntry = CurEntry;
//             CurEntry = CurEntry->LeftChild;
//         }
//     }
//
//     /* was it found? */
//     if (LastEntry == NULL)
//     {
//         /* release tunnel lock */
//         ExReleaseFastMutex(&Cache->Mutex);
//
//         return;
//     }
//
//     /* delete any matching key */
//     do
//     {
//         CurNode = CONTAINING_RECORD(LastEntry, TUNNEL_NODE_ENTRY, SplayInfo);
//
//         Successors = RtlRealSuccessor(LastEntry);
//         if (CurNode->DirectoryKey != DirectoryKey)
//         {
//             break;
//         }
//
//         /* remove from tunnel */
//         FsRtlRemoveNodeFromTunnel(Cache, CurNode, &PoolList, &Rebalance);
//         LastEntry = Successors;
//     }
//     while (LastEntry != NULL);
//
//     /* release tunnel lock */
//     ExReleaseFastMutex(&Cache->Mutex);
//
//     /* free pool */
//     FsRtlEmptyFreePoolList(&PoolList);
// }
//
// /*++
//  * @name FsRtlDeleteTunnelCache
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Cache
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlDeleteTunnelCache(IN PTUNNEL Cache)
// {
//     PLIST_ENTRY Entry, NextEntry;
//     PTUNNEL_NODE_ENTRY CurEntry;
//
//     PAGED_CODE();
//
//     /* check if tunnel cache is enabled */
//     if (!TunnelMaxEntries)
//     {
//         /* entries are disabled */
//         return;
//     }
//
//     /* free all entries */
//     Entry = Cache->TimerQueue.Flink;
//
//     while(Entry != &Cache->TimerQueue)
//     {
//         /* get node entry */
//         CurEntry = CONTAINING_RECORD(Entry, TUNNEL_NODE_ENTRY, TimerQueueEntry);
//
//         /* get next entry */
//          NextEntry = Entry->Flink;
//
//         /* remove entry from list */
//         RemoveEntryList(&CurEntry->TimerQueueEntry);
//
//         /* free entry */
//         FsRtlFreeTunnelNode(CurEntry, NULL);
//
//         /* move to next entry */
//         Entry = NextEntry;
//     }
//
//     /* reset object */
//     Cache->Cache = NULL;
//     Cache->NumEntries = 0;
//     InitializeListHead(&Cache->TimerQueue);
// }
//
// /*++
//  * @name FsRtlFindInTunnelCache
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Cache
//  *        FILLME
//  *
//  * @param DirectoryKey
//  *        FILLME
//  *
//  * @param ShortName
//  *        FILLME
//  *
//  * @param LongName
//  *        FILLME
//  *
//  * @param KeyByShortName
//  *        FILLME
//  *
//  * @param DataLength
//  *        FILLME
//  *
//  * @param Data
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlFindInTunnelCache(IN PTUNNEL Cache,
//                        IN ULONGLONG DirectoryKey,
//                        IN PUNICODE_STRING Name,
//                        OUT PUNICODE_STRING ShortName,
//                        OUT PUNICODE_STRING LongName,
//                        IN OUT PULONG DataLength,
//                        OUT PVOID Data)
// {
//     BOOLEAN Ret = FALSE;
//     PTUNNEL_NODE_ENTRY CurEntry;
//     LIST_ENTRY PoolList;
//     //NTSTATUS Status;
//     LONG Result;
//
//     PAGED_CODE();
//
//     /* check if tunnel cache is enabled */
//     if (!TunnelMaxEntries)
//     {
//         /* entries are disabled */
//         return FALSE;
//     }
//
//     /* initialize free pool list */
//     InitializeListHead(&PoolList);
//
//     /* acquire tunnel lock */
//     ExAcquireFastMutex(&Cache->Mutex);
//
//     /* prune old entries */
//     FsRtlPruneTunnelCache(Cache, &PoolList);
//
//     /* now search cache for existing entries */
//     CurEntry = (PTUNNEL_NODE_ENTRY)Cache->Cache;
//
//     while(CurEntry)
//     {
//         /* compare current node */
//         Result = FsRtlCompareNodeAndKey(CurEntry, DirectoryKey, Name);
//
//         if (Result > 0)
//         {
//             /* current directory key is bigger */
//             CurEntry = (PTUNNEL_NODE_ENTRY)CurEntry->SplayInfo.LeftChild;
//         }
//         else
//         {
//             if (Result == 0)
//             {
//                 /* found equal entry */
//                 break;
//             }
//
//             /* current directory key is smaller */
//             CurEntry = (PTUNNEL_NODE_ENTRY)CurEntry->SplayInfo.RightChild;
//         }
//     }
//
//     if (CurEntry != NULL)
//     {
//         _SEH2_TRY
//         {
//             /* copy short name */
//             RtlCopyUnicodeString(ShortName, &CurEntry->ShortName);
//
//             /* check size */
//             if (LongName->MaximumLength < CurEntry->LongName.Length)
//             {
//                 /* buffer is too small */
//                 LongName->Buffer = ExAllocatePool(PagedPool, CurEntry->LongName.Length);
//                 if (LongName->Buffer)
//                 {
//                     LongName->Length = CurEntry->LongName.Length;
//                     LongName->MaximumLength = CurEntry->LongName.MaximumLength;
//                     RtlMoveMemory(LongName->Buffer, CurEntry->LongName.Buffer, CurEntry->LongName.Length);
//                 }
//             }
//             else
//             {
//                 /* buffer is big enough */
//                 RtlCopyUnicodeString(LongName, &CurEntry->LongName);
//             }
//
//             /* copy data */
//             RtlMoveMemory(Data, CurEntry->Data, CurEntry->DataLength);
//
//             /* store size */
//             *DataLength = CurEntry->DataLength;
//
//             /* done */
//             Ret = TRUE;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Get the status */
//             //Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//     }
//
//     /* release tunnel lock */
//     ExReleaseFastMutex(&Cache->Mutex);
//
//     /* free pool */
//     FsRtlEmptyFreePoolList(&PoolList);
//
//     return Ret;
// }
//
// /*++
//  * @name FsRtlInitializeTunnelCache
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Cache
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlInitializeTunnelCache(IN PTUNNEL Cache)
// {
//     PAGED_CODE();
//
//     /* initialize mutex */
//     ExInitializeFastMutex(&Cache->Mutex);
//
//     /* initialize node tree */
//     Cache->Cache = NULL;
//
//     /* initialize timer list */
//     InitializeListHead(&Cache->TimerQueue);
//
//     /* initialize node count */
//     Cache->NumEntries = 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<TUNNEL_NODE_ENTRY>() == 112);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, SplayInfo) == 0);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, TimerQueueEntry) == 24);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, Time) == 40);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, DirectoryKey) == 48);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, Flags) == 56);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, LongName) == 64);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, ShortName) == 80);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, Data) == 96);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, DataLength) == 104);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<TUNNEL_NODE_ENTRY>() == 72);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, SplayInfo) == 0);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, TimerQueueEntry) == 12);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, Time) == 24);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, DirectoryKey) == 32);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, Flags) == 40);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, LongName) == 44);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, ShortName) == 52);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, Data) == 60);
    assert!(core::mem::offset_of!(TUNNEL_NODE_ENTRY, DataLength) == 64);
};
// END OFFSET CHECKS
