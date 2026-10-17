use crate::prelude::*;

// Symbols storage32.c reaches that the shared prelude surface does not carry yet.
// Same pattern (and same reason) as the extern blocks in ring/comcat.rs and
// ring/clipboard.rs: `crate::compobj` has its own hand-rolled FFI surface and is not
// reachable through `prelude`, so its helpers are bound at the symbol level; and the
// kernel32 imports below simply are not in `crate::win32` yet.
extern "system" {
    /// kernel32 (WINAPI) — not in crate::win32 yet.
    fn GetTickCount() -> DWORD;
    /// compobj.c (WINAPI) — not in crate::ole32_exports yet.
    fn ProgIDFromCLSID(clsid: REFCLSID, lplpszProgID: *mut LPOLESTR) -> HRESULT;
}
extern "C" {
    /// compobj.c's HKCR helper (compobj_private.h — plain C linkage, no WINAPI).
    fn open_classes_key(hkey: HKEY, name: LPCWSTR, access: REGSAM, retkey: *mut HKEY) -> LONG;
}

// Constants storage32.c matches on that are missing from crate::win_consts. They MUST be
// in scope: an unresolved name in a `match` arm is not an error in Rust — it silently
// becomes an irrefutable BINDING, so the first arm swallows every value and the rest of
// the match (including the `_` fallback) goes dead. That had already happened here for
// StgOpenStorageEx's `STGFMT_ANY` arm and StgOpenStorage's GetLastError() mapping.
// Values transcribed from the SDK headers, not from memory.
// OWED: hoist these into crate::win_consts (see the report).
/// sdk/include/psdk/objbase.h:935 `#define STGFMT_ANY 4`
/// sdk/include/psdk/winerror.h:227 `#define ERROR_FILE_NOT_FOUND 2`
/// sdk/include/psdk/winerror.h:228 `#define ERROR_PATH_NOT_FOUND 3`
/// sdk/include/psdk/winerror.h:257 `#define ERROR_SHARING_VIOLATION 32`

pub static STORAGE_magic: [u8; 8] = [0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];
pub static STORAGE_oldmagic: [u8; 8] = [0xd0, 0xcf, 0x11, 0xe0, 0x0e, 0x11, 0xfc, 0x0d];

#[repr(C)]
pub struct StorageInternalImpl {
    pub base: StorageBaseImpl,
    /// Entry in the parent's stream tracking list. The C is `struct list` (wine's
    /// intrusive list) — rustconv emitted the unrelated NT `LIST_ENTRY`.
    pub ParentListEntry: list,
    pub parentStorage: *mut StorageBaseImpl,
}
pub type PStorageInternalImpl = *mut StorageInternalImpl;

#[repr(C)]
pub struct TransactedDirEntry {
    pub transactedParentEntry: DirRef,
    pub inuse: BOOL,
    pub read: BOOL,
    pub dirty: BOOL,
    pub stream_dirty: BOOL,
    pub deleted: BOOL,
    pub stream_entry: DirRef,
    pub data: DirEntry,
    pub parent: DirRef,
    pub newTransactedParentEntry: DirRef,
}
pub type PTransactedDirEntry = *mut TransactedDirEntry;

#[repr(C)]
pub struct TransactedSnapshotImpl {
    pub base: StorageBaseImpl,
    pub scratch: *mut StorageBaseImpl,
    pub entries: *mut TransactedDirEntry,
    pub entries_size: ULONG,
    pub firstFreeEntry: ULONG,
    pub transactedParent: *mut StorageBaseImpl,
    pub lastTransactionSig: ULONG,
}
pub type PTransactedSnapshotImpl = *mut TransactedSnapshotImpl;

#[repr(C)]
pub struct TransactedSharedImpl {
    pub base: StorageBaseImpl,
    pub scratch: *mut TransactedSnapshotImpl,
    pub transactedParent: *mut StorageBaseImpl,
    pub lastTransactionSig: ULONG,
}
pub type PTransactedSharedImpl = *mut TransactedSharedImpl;

#[repr(C)]
pub struct BlockChainRun {
    pub firstSector: ULONG,
    pub firstOffset: ULONG,
    pub lastOffset: ULONG,
}
pub type PBlockChainRun = *mut BlockChainRun;

#[repr(C)]
pub struct BlockChainBlock {
    pub index: ULONG,
    pub sector: ULONG,
    pub read: BOOL,
    pub dirty: BOOL,
    pub data: [u8; MAX_BIG_BLOCK_SIZE as usize],
}
pub type PBlockChainBlock = *mut BlockChainBlock;

#[repr(C)]
pub struct BlockChainStream {
    pub parentStorage: *mut StorageImpl,
    pub headOfStreamPlaceHolder: *mut ULONG,
    pub ownerDirEntry: DirRef,
    pub indexCache: *mut BlockChainRun,
    pub indexCacheLen: ULONG,
    pub indexCacheSize: ULONG,
    pub cachedBlocks: [BlockChainBlock; 2],
    pub blockToEvict: ULONG,
    pub tailIndex: ULONG,
    pub numBlocks: ULONG,
}
pub type PBlockChainStream = *mut BlockChainStream;

#[repr(C)]
pub struct SmallBlockChainStream {
    pub parentStorage: *mut StorageImpl,
    pub ownerDirEntry: DirRef,
    pub headOfStreamPlaceHolder: *mut ULONG,
}
pub type PSmallBlockChainStream = *mut SmallBlockChainStream;

#[repr(C)]
pub struct IEnumSTATSTGImpl {
    pub IEnumSTATSTG_iface: IEnumSTATSTG,
    pub ref_: LONG,
    pub parentStorage: *mut StorageBaseImpl,
    pub storageDirEntry: DirRef,
    pub name: [u16; DIRENTRY_NAME_MAX_LEN],
}
pub type PIEnumSTATSTGImpl = *mut IEnumSTATSTGImpl;

pub const OLESTREAM_ID: u32 = 0x501;
pub const OLESTREAM_MAX_STR_LEN: usize = 255;

#[repr(C)]
pub struct OLECONVERT_OLESTREAM_DATA {
    pub dwOleID: DWORD,
    pub dwTypeID: DWORD,
    pub dwOleTypeNameLength: DWORD,
    pub strOleTypeName: [i8; OLESTREAM_MAX_STR_LEN],
    pub pstrOleObjFileName: *mut i8,
    pub dwOleObjFileNameLength: DWORD,
    pub dwMetaFileWidth: DWORD,
    pub dwMetaFileHeight: DWORD,
    pub strUnknown: [i8; 8],
    pub dwDataLength: DWORD,
    pub pData: *mut u8,
}
pub type POLECONVERT_OLESTREAM_DATA = *mut OLECONVERT_OLESTREAM_DATA;

#[repr(C)]
pub struct OLECONVERT_ISTORAGE_COMPOBJ {
    pub byUnknown1: [u8; 12],
    pub clsid: CLSID,
    pub dwCLSIDNameLength: DWORD,
    pub strCLSIDName: [i8; OLESTREAM_MAX_STR_LEN],
    pub dwOleTypeNameLength: DWORD,
    pub strOleTypeName: [i8; OLESTREAM_MAX_STR_LEN],
    pub dwProgIDNameLength: DWORD,
    pub strProgIDName: [i8; OLESTREAM_MAX_STR_LEN],
    pub byUnknown2: [u16; 16],
}
pub type POLECONVERT_ISTORAGE_COMPOBJ = *mut OLECONVERT_ISTORAGE_COMPOBJ;

#[repr(C)]
pub struct OLECONVERT_ISTORAGE_OLEPRES {
    pub byUnknown1: [u8; 28],
    pub dwExtentX: DWORD,
    pub dwExtentY: DWORD,
    pub dwSize: DWORD,
    pub pData: *mut u8,
}
pub type POLECONVERT_ISTORAGE_OLEPRES = *mut OLECONVERT_ISTORAGE_OLEPRES;

pub const OleStream_LinkedObject: u32 = 0x00000001;
pub const OleStream_Convert: u32 = 0x00000004;

unsafe fn BlockChainStream_GetCount(This: *mut BlockChainStream) -> ULONG {
    (*This).numBlocks
}

pub unsafe fn validateSTGM(stgm: u32) -> HRESULT {
    let access = STGM_ACCESS_MODE(stgm);
    let share = STGM_SHARE_MODE(stgm);
    let create = STGM_CREATE_MODE(stgm);

    if (stgm & !STGM_KNOWN_FLAGS) != 0 {
        ERR(b"unknown flags %08x\n\0".as_ptr() as LPCSTR);
        return E_FAIL;
    }

    match access {
        STGM_READ | STGM_WRITE | STGM_READWRITE => {},
        _ => return E_FAIL,
    }

    match share {
        STGM_SHARE_DENY_NONE | STGM_SHARE_DENY_READ | STGM_SHARE_DENY_WRITE | STGM_SHARE_EXCLUSIVE => {},
        0 => {
            if (stgm & STGM_TRANSACTED) == 0 {
                return E_FAIL;
            }
        },
        _ => return E_FAIL,
    }

    match create {
        STGM_CREATE | STGM_FAILIFTHERE => {},
        _ => return E_FAIL,
    }

    if (stgm & STGM_TRANSACTED) != 0 && (stgm & STGM_SIMPLE) != 0 {
        return E_FAIL;
    }

    if create == STGM_CREATE && (stgm & STGM_CONVERT) != 0 {
        return E_FAIL;
    }

    if (stgm & STGM_NOSCRATCH) != 0 && (stgm & STGM_TRANSACTED) == 0 {
        return E_FAIL;
    }

    if (stgm & STGM_NOSNAPSHOT) != 0 &&
       ((stgm & STGM_TRANSACTED) == 0 ||
        share == STGM_SHARE_EXCLUSIVE ||
        share == STGM_SHARE_DENY_WRITE) {
        return E_FAIL;
    }

    S_OK
}

unsafe fn GetShareModeFromSTGM(stgm: DWORD) -> DWORD {
    match STGM_SHARE_MODE(stgm) {
        0 => {
            assert(stgm & STGM_TRANSACTED != 0);
            FILE_SHARE_READ | FILE_SHARE_WRITE
        }
        STGM_SHARE_DENY_NONE => FILE_SHARE_READ | FILE_SHARE_WRITE,
        STGM_SHARE_DENY_READ => FILE_SHARE_WRITE,
        STGM_SHARE_DENY_WRITE | STGM_SHARE_EXCLUSIVE => FILE_SHARE_READ,
        _ => {
            ERR(b"Invalid share mode!\n\0".as_ptr() as PCHAR);
            assert(false);
            0
        }
    }
}

unsafe fn GetAccessModeFromSTGM(stgm: DWORD) -> DWORD {
    match STGM_ACCESS_MODE(stgm) {
        STGM_READ => return GENERIC_READ,
        STGM_WRITE | STGM_READWRITE => return GENERIC_READ | GENERIC_WRITE,
        _ => {}
    }
    ERR(b"Invalid access mode!\n\0".as_ptr() as PCHAR);
    assert(0);
    0
}

unsafe fn GetCreationModeFromSTGM(stgm: DWORD) -> DWORD {
    match STGM_CREATE_MODE(stgm) {
        STGM_CREATE => CREATE_ALWAYS,
        STGM_CONVERT => {
            FIXME(b"STGM_CONVERT not implemented!\n\0".as_ptr() as PCHAR);
            CREATE_NEW
        }
        STGM_FAILIFTHERE => CREATE_NEW,
        _ => {
            ERR(b"Invalid create mode!\n\0".as_ptr() as PCHAR);
            assert(0);
            0
        }
    }
}

unsafe fn impl_from_IDirectWriterLock(iface: *mut IDirectWriterLock) -> *mut StorageBaseImpl {
    (iface as *mut u8).sub(
        core::mem::offset_of!(StorageBaseImpl, IDirectWriterLock_iface)
    ) as *mut StorageBaseImpl
}

unsafe fn entryNameCmp(
    name1: *const u16,
    name2: *const u16) -> i32
{
    

    let mut diff: i32 = lstrlenW(name1) - lstrlenW(name2);

    let mut n1 = name1;
    let mut n2 = name2;

    while diff == 0 && *n1 != 0 {
        // The C subtracts two wint_t promoted to int: widen first, or u16 wraps.
        diff = towupper(*n1 as u16) as c_int - towupper(*n2 as u16) as c_int;
        n1 = n1.offset(1);
        n2 = n2.offset(1);
    }

    return diff;
}

unsafe fn setEntryLink(entry: *mut DirEntry, relation: ULONG, new_target: DirRef) {
    match relation {
        DIRENTRY_RELATION_PREVIOUS => {
            (*entry).leftChild = new_target;
        }
        DIRENTRY_RELATION_NEXT => {
            (*entry).rightChild = new_target;
        }
        DIRENTRY_RELATION_DIR => {
            (*entry).dirRootEntry = new_target;
        }
        _ => {
            assert(false);
        }
    }
}

unsafe fn impl_from_IEnumSTATSTG(iface: *mut IEnumSTATSTG) -> *mut IEnumSTATSTGImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(IEnumSTATSTGImpl, IEnumSTATSTG_iface) as isize)) as *mut IEnumSTATSTGImpl
}

stdmethod! {
fn directwriterlock_QueryInterface(
    iface: *mut IDirectWriterLock,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IDirectWriterLock(iface);
    IStorage_QueryInterface(
        core::ptr::addr_of_mut!((*This).IStorage_iface),
        riid,
        obj,
    )
}
}

stdmethod! {
fn directwriterlock_AddRef(iface: *mut IDirectWriterLock) -> ULONG {
    let This = impl_from_IDirectWriterLock(iface);
    IStorage_AddRef(core::ptr::addr_of_mut!((*This).IStorage_iface))
}
}

stdmethod! {
fn directwriterlock_Release(iface: *mut IDirectWriterLock) -> ULONG {
    let This = impl_from_IDirectWriterLock(iface);
    IStorage_Release(core::ptr::addr_of_mut!((*This).IStorage_iface))
}
}

stdmethod! {
fn directwriterlock_WaitForWriteAccess(iface: *mut IDirectWriterLock, timeout: DWORD) -> HRESULT {
    let _This = impl_from_IDirectWriterLock(iface);
    FIXME(b"(%p)->(%d): stub\n\0".as_ptr() as LPCSTR);
    E_NOTIMPL
}
}

stdmethod! {
fn directwriterlock_ReleaseWriteAccess(iface: *mut IDirectWriterLock) -> HRESULT {
    let _This = impl_from_IDirectWriterLock(iface);
    FIXME(b"(%p): stub\n\0".as_ptr() as PCHAR);
    E_NOTIMPL
}
}

stdmethod! {
fn directwriterlock_HaveWriteAccess(iface: *mut IDirectWriterLock) -> HRESULT {
    let _This = impl_from_IDirectWriterLock(iface);
    FIXME(b"(%p): stub\n\0".as_ptr() as PCHAR);
    E_NOTIMPL
}
}

unsafe fn findElement(
    storage: *mut StorageBaseImpl,
    storageEntry: u32,
    name: *const u16,
    data: *mut DirEntry,
) -> u32 {
    let mut currentEntry: u32;

    StorageBaseImpl_ReadDirEntry(storage, storageEntry, data);

    currentEntry = (*data).dirRootEntry;

    while currentEntry != 0xFFFFFFFF {
        let cmp: i32;

        StorageBaseImpl_ReadDirEntry(storage, currentEntry, data);

        cmp = entryNameCmp(name, (*data).name.as_ptr());

        if cmp == 0 {
            break;
        } else if cmp < 0 {
            currentEntry = (*data).leftChild;
        } else if cmp > 0 {
            currentEntry = (*data).rightChild;
        }
    }

    currentEntry
}

unsafe fn findTreeParent(
    storage: *mut StorageBaseImpl,
    storageEntry: DirRef,
    childName: *const u16,
    parentData: *mut DirEntry,
    parentEntry: *mut DirRef,
    relation: *mut u32,
) -> i32 {
    let mut childEntry: DirRef;
    let mut childData: DirEntry = core::mem::zeroed();

    StorageBaseImpl_ReadDirEntry(storage, storageEntry, parentData);

    *parentEntry = storageEntry;
    *relation = DIRENTRY_RELATION_DIR;

    childEntry = (*parentData).dirRootEntry;

    while childEntry != DIRENTRY_NULL {
        let cmp: i32;

        StorageBaseImpl_ReadDirEntry(storage, childEntry, &mut childData);

        cmp = entryNameCmp(childName, childData.name.as_ptr());

        if cmp == 0 {
            break;
        } else if cmp < 0 {
            *parentData = childData;
            *parentEntry = childEntry;
            *relation = DIRENTRY_RELATION_PREVIOUS;

            childEntry = (*parentData).leftChild;
        } else if cmp > 0 {
            *parentData = childData;
            *parentEntry = childEntry;
            *relation = DIRENTRY_RELATION_NEXT;

            childEntry = (*parentData).rightChild;
        }
    }

    if childEntry == DIRENTRY_NULL {
        STG_E_FILENOTFOUND
    } else {
        S_OK
    }
}

unsafe fn insertIntoTree(
    This: *mut StorageBaseImpl,
    parentStorageIndex: DirRef,
    newEntryIndex: DirRef) -> HRESULT
{
    let mut currentEntry: DirEntry = core::mem::zeroed();
    let mut newEntry: DirEntry = core::mem::zeroed();

    StorageBaseImpl_ReadDirEntry(
        This,
        newEntryIndex,
        &mut newEntry
    );

    StorageBaseImpl_ReadDirEntry(
        This,
        parentStorageIndex,
        &mut currentEntry
    );

    if currentEntry.dirRootEntry != DIRENTRY_NULL {
        let mut found: BOOL = 0;
        let mut current: DirRef;
        let mut next: DirRef;
        let mut previous: DirRef;
        let currentEntryId: DirRef;

        currentEntryId = currentEntry.dirRootEntry;

        StorageBaseImpl_ReadDirEntry(
            This,
            currentEntry.dirRootEntry,
            &mut currentEntry
        );

        previous = currentEntry.leftChild;
        next = currentEntry.rightChild;
        current = currentEntryId;

        while found == 0 {
            let diff: i32 = entryNameCmp(
                newEntry.name.as_ptr(),
                currentEntry.name.as_ptr()
            );

            if diff < 0 {
                if previous != DIRENTRY_NULL {
                    StorageBaseImpl_ReadDirEntry(
                        This,
                        previous,
                        &mut currentEntry
                    );
                    current = previous;
                } else {
                    currentEntry.leftChild = newEntryIndex;
                    StorageBaseImpl_WriteDirEntry(
                        This,
                        current,
                        &mut currentEntry
                    );
                    found = 1;
                }
            } else if diff > 0 {
                if next != DIRENTRY_NULL {
                    StorageBaseImpl_ReadDirEntry(
                        This,
                        next,
                        &mut currentEntry
                    );
                    current = next;
                } else {
                    currentEntry.rightChild = newEntryIndex;
                    StorageBaseImpl_WriteDirEntry(
                        This,
                        current,
                        &mut currentEntry
                    );
                    found = 1;
                }
            } else {
                return STG_E_FILEALREADYEXISTS;
            }

            previous = currentEntry.leftChild;
            next = currentEntry.rightChild;
        }
    } else {
        currentEntry.dirRootEntry = newEntryIndex;
        StorageBaseImpl_WriteDirEntry(
            This,
            parentStorageIndex,
            &mut currentEntry
        );
    }

    S_OK
}

unsafe fn IEnumSTATSTGImpl_GetNextRef(
    This: *mut IEnumSTATSTGImpl,
    ref_: *mut DirRef) -> HRESULT
{
    let mut result: DirRef = DIRENTRY_NULL;
    let mut searchNode: DirRef;
    let mut entry: DirEntry = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut result_name: [u16; DIRENTRY_NAME_MAX_LEN] = [0; DIRENTRY_NAME_MAX_LEN];

    TRACE!(b"%p,%p\n\0".as_ptr() as LPCSTR);

    hr = StorageBaseImpl_ReadDirEntry(
        (*This).parentStorage,
        (*(*This).parentStorage).storageDirEntry,
        &mut entry
    );
    searchNode = entry.dirRootEntry;

    while SUCCEEDED(hr) && searchNode != DIRENTRY_NULL {
        hr = StorageBaseImpl_ReadDirEntry((*This).parentStorage, searchNode, &mut entry);

        if SUCCEEDED(hr) {
            let diff: i32 = entryNameCmp(entry.name.as_ptr(), (*This).name.as_ptr());

            if diff <= 0 {
                searchNode = entry.rightChild;
            } else {
                result = searchNode;
                core::ptr::copy_nonoverlapping(
                    entry.name.as_ptr(),
                    result_name.as_mut_ptr(),
                    DIRENTRY_NAME_MAX_LEN
                );
                searchNode = entry.leftChild;
            }
        }
    }

    if SUCCEEDED(hr) {
        *ref_ = result;
        if result != DIRENTRY_NULL {
            core::ptr::copy_nonoverlapping(
                result_name.as_ptr(),
                (*This).name.as_mut_ptr(),
                DIRENTRY_NAME_MAX_LEN
            );
        }
    }

    TRACE!(b"<-- %08x\n\0".as_ptr() as LPCSTR);
    hr
}

unsafe fn IEnumSTATSTGImpl_Destroy(This: *mut IEnumSTATSTGImpl) {
    IStorage_Release(&mut (*(*This).parentStorage).IStorage_iface as *mut _ as *mut IStorage);
    HeapFree(GetProcessHeap(), 0, This as LPVOID);
}

stdmethod! {
fn IEnumSTATSTGImpl_QueryInterface(
    iface: *mut IEnumSTATSTG,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    

    let This = impl_from_IEnumSTATSTG(iface);

    TRACE!(b"%p,%s,%p\n\0".as_ptr() as LPCSTR);

    if ppvObject == core::ptr::null_mut() {
        return 0x80070057u32 as HRESULT;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualGUID(&IID_IUnknown, riid) || IsEqualGUID(&IID_IEnumSTATSTG, riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IEnumSTATSTG_iface) as *mut core::ffi::c_void;
        IEnumSTATSTG_AddRef(core::ptr::addr_of_mut!((*This).IEnumSTATSTG_iface));
        TRACE!(b"<-- %p\n\0".as_ptr() as LPCSTR);
        return 0u32 as HRESULT;
    }

    TRACE!(b"<-- E_NOINTERFACE\n\0".as_ptr() as LPCSTR);
    0x80004002u32 as HRESULT
}
}

stdmethod! {
fn IEnumSTATSTGImpl_AddRef(iface: *mut IEnumSTATSTG) -> ULONG {
    let This = impl_from_IEnumSTATSTG(iface);
    InterlockedIncrement(&mut (*This).ref_) as ULONG
}
}

stdmethod! {
fn IEnumSTATSTGImpl_Reset(
    iface: *mut IEnumSTATSTG
) -> HRESULT {
    let This = impl_from_IEnumSTATSTG(iface);

    TRACE!(b"%p\n\0".as_ptr() as LPCSTR);

    if (*(*This).parentStorage).reverted != 0 {
        TRACE!(b"<-- STG_E_REVERTED\n\0".as_ptr() as LPCSTR);
        return STG_E_REVERTED;
    }

    (*This).name[0] = 0;

    S_OK
}
}

stdmethod! {
fn IEnumSTATSTGImpl_Release(
    iface: *mut IEnumSTATSTG
) -> ULONG {
    let This = impl_from_IEnumSTATSTG(iface);

    let newRef = InterlockedDecrement(&mut (*This).ref_);

    if newRef == 0 {
        IEnumSTATSTGImpl_Destroy(This);
    }

    newRef as ULONG
}
}

stdmethod! {
fn IEnumSTATSTGImpl_Clone(
    iface: *mut IEnumSTATSTG,
    ppenum: *mut *mut IEnumSTATSTG,
) -> HRESULT {
    let This = impl_from_IEnumSTATSTG(iface);

    if (*(*This).parentStorage).reverted != 0 {
        return STG_E_REVERTED;
    }

    if ppenum.is_null() {
        return E_INVALIDARG;
    }

    let newClone = IEnumSTATSTGImpl_Construct(
        (*This).parentStorage,
        (*This).storageDirEntry,
    );
    if newClone.is_null() {
        *ppenum = core::ptr::null_mut();
        return E_OUTOFMEMORY;
    }

    core::ptr::copy_nonoverlapping(
        (*This).name.as_ptr(),
        (*newClone).name.as_mut_ptr(),
        (*newClone).name.len(),
    );

    *ppenum = core::ptr::addr_of_mut!((*newClone).IEnumSTATSTG_iface);

    S_OK
}
}

stdmethod! {
fn StorageBaseImpl_QueryInterface(
    iface: *mut IStorage,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IStorage(iface);

    TRACE!(b"%p,%s,%p\n\0".as_ptr() as PCHAR);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualGUID(core::ptr::addr_of!(IID_IUnknown), riid) ||
       IsEqualGUID(core::ptr::addr_of!(IID_IStorage), riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).IStorage_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualGUID(core::ptr::addr_of!(IID_IPropertySetStorage), riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).IPropertySetStorage_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualGUID(core::ptr::addr_of!(IID_IDirectWriterLock), riid) && (*This).lockingrole == SWMR_Writer
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).IDirectWriterLock_iface) as *mut core::ffi::c_void;
    }
    else
    {
        TRACE!(b"<-- E_NOINTERFACE\n\0".as_ptr() as PCHAR);
        return E_NOINTERFACE;
    }

    IStorage_AddRef(iface);
    TRACE!(b"<-- %p\n\0".as_ptr() as PCHAR);
    return S_OK;
}
}

stdmethod! {
fn StorageBaseImpl_AddRef(iface: *mut IStorage) -> u32 {
    let This = impl_from_IStorage(iface);
    let ref_ = InterlockedIncrement(&mut (*This).ref_);
    
    TRACE!(b"(%p) AddRef to %d\n\0".as_ptr() as PCHAR);
    
    ref_ as u32
}
}

stdmethod! {
fn StorageBaseImpl_Release(iface: *mut IStorage) -> ULONG {
    let This = impl_from_IStorage(iface);

    let ref_count = InterlockedDecrement(&mut (*This).ref_);

    

    TRACE!(b"(%p) ReleaseRef to %d\n\0".as_ptr() as *const c_char);

    if ref_count == 0 {
        StorageBaseImpl_Destroy(This);
    }

    ref_count as ULONG
}
}

stdmethod! {
fn StorageBaseImpl_EnumElements(
    iface: *mut IStorage,
    reserved1: DWORD,
    reserved2: *mut core::ffi::c_void,
    reserved3: DWORD,
    ppenum: *mut *mut IEnumSTATSTG,
) -> HRESULT {
    let This = impl_from_IStorage(iface);
    let newEnum: *mut IEnumSTATSTGImpl;

    TRACE!(b"(%p, %d, %p, %d, %p)\n\0".as_ptr() as LPCSTR);

    if ppenum.is_null() {
        return E_INVALIDARG;
    }

    if (*This).reverted != 0 {
        return STG_E_REVERTED;
    }

    newEnum = IEnumSTATSTGImpl_Construct(This, (*This).storageDirEntry);

    if !newEnum.is_null() {
        *ppenum = core::ptr::addr_of_mut!((*newEnum).IEnumSTATSTG_iface);
        return S_OK;
    }

    E_OUTOFMEMORY
}
}

stdmethod! {
fn StorageBaseImpl_SetClass(
    iface: *mut IStorage,
    clsid: *const CLSID,
) -> HRESULT {
    let This = impl_from_IStorage(iface);
    let mut hRes: HRESULT;
    let mut currentEntry: DirEntry = core::mem::zeroed();

    

    TRACE!(b"(%p, %s)\n\0".as_ptr() as *const c_char);

    if (*This).reverted != 0 {
        return STG_E_REVERTED;
    }

    hRes = StorageBaseImpl_ReadDirEntry(
        This,
        (*This).storageDirEntry,
        &mut currentEntry,
    );

    if SUCCEEDED(hRes) {
        currentEntry.clsid = *clsid;

        hRes = StorageBaseImpl_WriteDirEntry(
            This,
            (*This).storageDirEntry,
            &currentEntry,
        );
    }

    if SUCCEEDED(hRes) {
        hRes = StorageBaseImpl_Flush(This);
    }

    hRes
}
}

unsafe fn IEnumSTATSTGImpl_Construct(
    parentStorage: *mut StorageBaseImpl,
    storageDirEntry: DirRef,
) -> *mut IEnumSTATSTGImpl {
    let newEnumeration: *mut IEnumSTATSTGImpl;

    newEnumeration = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<IEnumSTATSTGImpl>()) as SIZE_T) as *mut IEnumSTATSTGImpl;

    if !newEnumeration.is_null() {
        (*newEnumeration).IEnumSTATSTG_iface.lpVtbl = &IEnumSTATSTGImpl_Vtbl as *const IEnumSTATSTGVtbl as *mut IEnumSTATSTGVtbl;
        (*newEnumeration).ref_ = 1;
        (*newEnumeration).name[0] = 0;

        (*newEnumeration).parentStorage = parentStorage;
        IStorage_AddRef(core::ptr::addr_of_mut!((*(*newEnumeration).parentStorage).IStorage_iface));

        (*newEnumeration).storageDirEntry = storageDirEntry;
    }

    newEnumeration
}

stdmethod! {
fn StorageBaseImpl_Commit(
    iface: *mut IStorage,
    grfCommitFlags: DWORD,
) -> HRESULT {
    let This = impl_from_IStorage(iface);
    TRACE!(b"(%p %d)\n\0".as_ptr() as PCHAR);
    StorageBaseImpl_Flush(This)
}
}

stdmethod! {
fn StorageBaseImpl_SetStateBits(
    iface: *mut IStorage,
    grfStateBits: DWORD,
    grfMask: DWORD,
) -> HRESULT {
    let This = impl_from_IStorage(iface);

    if (*This).reverted != 0 {
        return STG_E_REVERTED;
    }

    (*This).stateBits = ((*This).stateBits & !grfMask) | (grfStateBits & grfMask);
    S_OK
}
}

stdmethod! {
fn StorageInternalImpl_Commit(
    iface: *mut IStorage,
    grfCommitFlags: DWORD,
) -> HRESULT {
    let This = impl_from_IStorage(iface);
    TRACE!(b"(%p,%x)\n\0".as_ptr() as LPCSTR);
    StorageBaseImpl_Flush(This)
}
}

unsafe extern "C" fn StorageImpl_Invalidate(iface: *mut StorageBaseImpl) {
    let This = iface as *mut StorageImpl;

    StorageBaseImpl_DeleteAll(&mut (*This).base);

    (*This).base.reverted = 1;
}

unsafe extern "C" fn StorageInternalImpl_Invalidate(base: *mut StorageBaseImpl) {
    let This = base as *mut StorageInternalImpl;

    if (*This).base.reverted == 0 {
        TRACE!(b"Storage invalidated (stg=%p)\n\0".as_ptr() as PCHAR);

        (*This).base.reverted = 1;

        (*This).parentStorage = core::ptr::null_mut();

        StorageBaseImpl_DeleteAll(&mut (*This).base);

        list_remove(&mut (*This).ParentListEntry);
    }
}

unsafe extern "C" fn TransactedSnapshotImpl_Invalidate(This: *mut StorageBaseImpl) {
    if (*This).reverted == 0 {
        TRACE!(b"Storage invalidated (stg=%p)\n\0".as_ptr() as PCHAR);
        
        (*This).reverted = 1;
        
        StorageBaseImpl_DeleteAll(This);
    }
}

unsafe extern "C" fn TransactedSharedImpl_Invalidate(This: *mut StorageBaseImpl) {
    if (*This).reverted == 0 {
        
        TRACE!(b"Storage invalidated (stg=%p)\n\0".as_ptr() as *const c_char);

        (*This).reverted = 1;

        StorageBaseImpl_DeleteAll(This);
    }
}

stdmethod! {
fn TransactedSharedImpl_Revert(
    iface: *mut IStorage
) -> HRESULT {
    let This = impl_from_IStorage(iface) as *mut TransactedSharedImpl;

    TRACE!(b"(%p)\n\0".as_ptr() as LPCSTR);

    StorageBaseImpl_DeleteAll(&mut (*This).base);

    IStorage_Revert(&mut (*(*This).scratch).base.IStorage_iface)
}
}

unsafe fn StorageBaseImpl_DupStorageTree(
    dst: *mut StorageBaseImpl,
    dst_entry: *mut DirRef,
    src: *mut StorageBaseImpl,
    src_entry: DirRef,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut data: DirEntry = core::mem::zeroed();
    let mut has_stream: BOOL = 0;

    if src_entry == DIRENTRY_NULL {
        *dst_entry = DIRENTRY_NULL;
        return S_OK;
    }

    hr = StorageBaseImpl_ReadDirEntry(src, src_entry, &mut data);
    if SUCCEEDED(hr) {
        has_stream = (data.stgType as DWORD == STGTY_STREAM && data.size.QuadPart != 0) as BOOL;
        data.startingBlock = BLOCK_END_OF_CHAIN;
        data.size.QuadPart = 0;

        hr = StorageBaseImpl_DupStorageTree(dst, &mut data.leftChild, src, data.leftChild);
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_DupStorageTree(dst, &mut data.rightChild, src, data.rightChild);
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_DupStorageTree(dst, &mut data.dirRootEntry, src, data.dirRootEntry);
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_CreateDirEntry(dst, &data, dst_entry);
    }

    if SUCCEEDED(hr) && has_stream != 0 {
        hr = StorageBaseImpl_CopyStream(dst, *dst_entry, src, src_entry);
    }

    hr
}

unsafe fn StorageImpl_ReadBigBlock(
    This: *mut StorageImpl,
    blockIndex: ULONG,
    buffer: *mut core::ffi::c_void,
    out_read: *mut ULONG,
) -> HRESULT {
    let mut ulOffset: ULARGE_INTEGER = core::mem::zeroed();
    let mut read: DWORD = 0;
    let hr: HRESULT;

    ulOffset.QuadPart = StorageImpl_GetBigBlockOffset(This, blockIndex);

    hr = StorageImpl_ReadAt(This, ulOffset, buffer, (*This).bigBlockSize, &mut read);

    if hr >= 0 && read < (*This).bigBlockSize {
        core::ptr::write_bytes(
            (buffer as *mut u8).add(read as usize),
            0,
            ((*This).bigBlockSize - read) as usize,
        );
    }

    if !out_read.is_null() {
        *out_read = read;
    }

    hr
}

#[inline]
unsafe fn impl_from_IStorage(iface: *mut IStorage) -> *mut StorageBaseImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(StorageBaseImpl, IStorage_iface) as isize)) as *mut StorageBaseImpl
}

unsafe fn StorageImpl_ReadDWordFromBigBlock(
    This: *mut StorageImpl,
    blockIndex: ULONG,
    offset: ULONG,
    value: *mut DWORD,
) -> BOOL {
    let mut ulOffset: ULARGE_INTEGER = core::mem::zeroed();
    let mut read: DWORD = 0;
    let mut tmp: DWORD = 0;

    ulOffset.QuadPart = StorageImpl_GetBigBlockOffset(This, blockIndex);
    ulOffset.QuadPart += offset as ULONGLONG;

    StorageImpl_ReadAt(
        This,
        ulOffset,
        core::ptr::addr_of_mut!(tmp) as *mut core::ffi::c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(read),
    );
    
    
    
    *value = lendian32toh(tmp);
    (read == core::mem::size_of::<DWORD>() as DWORD) as BOOL
}

unsafe fn StorageImpl_WriteBigBlock(
    This: *mut StorageImpl,
    blockIndex: ULONG,
    buffer: *const core::ffi::c_void,
) -> BOOL {
    let mut ulOffset: ULARGE_INTEGER = core::mem::zeroed();
    let mut wrote: DWORD = 0;

    ulOffset.QuadPart = StorageImpl_GetBigBlockOffset(This, blockIndex);

    StorageImpl_WriteAt(This, ulOffset, buffer, (*This).bigBlockSize, &mut wrote);
    (wrote == (*This).bigBlockSize) as BOOL
}

unsafe fn StorageImpl_WriteDWordToBigBlock(
    This: *mut StorageImpl,
    blockIndex: ULONG,
    offset: ULONG,
    value: DWORD,
) -> BOOL {
    

    let mut ulOffset: ULARGE_INTEGER = core::mem::zeroed();
    let mut wrote: DWORD = 0;

    ulOffset.QuadPart = StorageImpl_GetBigBlockOffset(This, blockIndex);
    ulOffset.QuadPart += offset as u64;

    let value = htole32(value);
    StorageImpl_WriteAt(
        This,
        ulOffset,
        core::ptr::addr_of!(value) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(wrote),
    );
    (wrote == core::mem::size_of::<DWORD>() as ULONG) as BOOL
}

unsafe fn StorageImpl_LockOne(This: *mut StorageImpl, start: ULONG, end: ULONG) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut i: i32;
    let mut j: i32;
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut cb: ULARGE_INTEGER = core::mem::zeroed();

    cb.QuadPart = 1;

    i = start as i32;
    while i <= end as i32 {
        offset.QuadPart = i as ULONGLONG;
        hr = StorageImpl_LockRegion(This, offset, cb, LOCK_ONLYONCE, core::ptr::null_mut());
        if hr != STG_E_ACCESSDENIED && hr != STG_E_LOCKVIOLATION {
            break;
        }
        i += 1;
    }

    if SUCCEEDED(hr) {
        j = 0;
        while j < ((*This).locked_bytes.len() as i32) {
            if (*This).locked_bytes[j as usize] == 0 {
                (*This).locked_bytes[j as usize] = i as ULONG;
                break;
            }
            j += 1;
        }
    }

    hr
}

unsafe fn StorageImpl_LockRegionSync(
    This: *mut StorageImpl,
    offset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
    supported: *mut BOOL,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut delay: i32 = 0;
    let start_time: DWORD = GetTickCount();
    let mut last_sanity_check: DWORD = start_time;
    let mut sanity_offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut sanity_cb: ULARGE_INTEGER = core::mem::zeroed();

    sanity_offset.QuadPart = RANGELOCK_UNK1_FIRST as ULONGLONG;
    sanity_cb.QuadPart = (RANGELOCK_UNK1_LAST - RANGELOCK_UNK1_FIRST + 1) as ULONGLONG;

    loop {
        hr = StorageImpl_LockRegion(This, offset, cb, dwLockType, supported);

        if hr == STG_E_ACCESSDENIED || hr == STG_E_LOCKVIOLATION {
            let current_time: DWORD = GetTickCount();
            if current_time.wrapping_sub(start_time) >= 20000 {
                break;
            }
            if current_time.wrapping_sub(last_sanity_check) >= 500 {
                hr = StorageImpl_LockRegion(This, sanity_offset, sanity_cb, WINE_LOCK_READ, core::ptr::null_mut());
                if hr == STG_E_ACCESSDENIED || hr == STG_E_LOCKVIOLATION {
                    break;
                }
                if SUCCEEDED(hr) {
                    StorageImpl_UnlockRegion(This, sanity_offset, sanity_cb, WINE_LOCK_READ);
                    hr = STG_E_ACCESSDENIED;
                }

                last_sanity_check = current_time;
            }
            Sleep(delay as DWORD);
            if delay < 150 {
                delay += 1;
            }
        }
        // The C's `do { .. } while (hr == STG_E_ACCESSDENIED || hr == STG_E_LOCKVIOLATION)`:
        // re-tested AFTER the sanity probe, so a probe failing some other way ends the wait
        // with that error instead of retrying the full 20 seconds.
        if !(hr == STG_E_ACCESSDENIED || hr == STG_E_LOCKVIOLATION) {
            break;
        }
    }

    hr
}

unsafe extern "C" fn StorageImpl_UnlockTransaction(base: *mut StorageBaseImpl, write: BOOL) -> HRESULT {
    let This = base as *mut StorageImpl;
    let hr: HRESULT;
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut cb: ULARGE_INTEGER = core::mem::zeroed();

    if write != 0 {
        offset.QuadPart = RANGELOCK_TRANSACTION_FIRST as ULONGLONG;
        cb.QuadPart = (RANGELOCK_TRANSACTION_LAST - RANGELOCK_TRANSACTION_FIRST + 1) as ULONGLONG;
    } else {
        offset.QuadPart = RANGELOCK_COMMIT as ULONGLONG;
        cb.QuadPart = 1;
    }

    hr = StorageImpl_UnlockRegion(This, offset, cb, LOCK_ONLYONCE);

    hr
}

unsafe fn StorageImpl_CheckLockRange(
    This: *mut StorageImpl,
    start: ULONG,
    end: ULONG,
    fail_hr: HRESULT,
) -> HRESULT {
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut cb: ULARGE_INTEGER = core::mem::zeroed();

    offset.QuadPart = start as u64;
    cb.QuadPart = (1 + end - start) as u64;

    let hr = StorageImpl_LockRegion(This, offset, cb, LOCK_ONLYONCE, core::ptr::null_mut());
    if hr >= 0 {
        StorageImpl_UnlockRegion(This, offset, cb, LOCK_ONLYONCE);
    }

    if hr < 0 {
        fail_hr
    } else {
        S_OK
    }
}

unsafe fn TransactedSnapshotImpl_CreateStubEntry(
    This: *mut TransactedSnapshotImpl,
    parentEntryRef: DirRef,
) -> DirRef {
    let stubEntryRef: DirRef;
    let entry: *mut TransactedDirEntry;

    stubEntryRef = TransactedSnapshotImpl_FindFreeEntry(This);

    if stubEntryRef != DIRENTRY_NULL {
        entry = (*This).entries.offset((stubEntryRef as usize) as isize);

        (*entry).newTransactedParentEntry = parentEntryRef;
        (*entry).transactedParentEntry = parentEntryRef;

        (*entry).read = 0;
    }

    stubEntryRef
}

unsafe extern "C" fn TransactedSnapshotImpl_CreateDirEntry(
    base: *mut StorageBaseImpl,
    newData: *const DirEntry,
    index: *mut DirRef,
) -> HRESULT {
    let This = base as *mut TransactedSnapshotImpl;
    let new_ref: DirRef;
    let new_entry: *mut TransactedDirEntry;

    new_ref = TransactedSnapshotImpl_FindFreeEntry(This);
    if new_ref == DIRENTRY_NULL {
        return E_OUTOFMEMORY;
    }

    new_entry = (*This).entries.offset((new_ref as usize) as isize);

    (*new_entry).newTransactedParentEntry = DIRENTRY_NULL;
    (*new_entry).transactedParentEntry = DIRENTRY_NULL;
    (*new_entry).read = 1;
    (*new_entry).dirty = 1;
    core::ptr::copy_nonoverlapping(
        newData,
        &mut (*new_entry).data,
        1,
    );

    *index = new_ref;

    TRACE!(b"%s l=%x r=%x d=%x <-- %x\n\0".as_ptr() as LPCSTR);

    S_OK
}

unsafe fn TransactedSnapshotImpl_FindNextChild(
    This: *mut TransactedSnapshotImpl,
    current: DirRef,
) -> DirRef {
    let parent: DirRef;
    let parent_entry: *mut TransactedDirEntry;

    parent = (*(*This).entries.offset(current as isize)).parent;
    parent_entry = &mut (*(*This).entries.offset(parent as isize));

    if parent != DIRENTRY_NULL && (*parent_entry).data.dirRootEntry != current {
        if (*parent_entry).data.rightChild != current && (*parent_entry).data.rightChild != DIRENTRY_NULL {
            (*(*This).entries.offset((*parent_entry).data.rightChild as isize)).parent = parent;
            return TransactedSnapshotImpl_FindFirstChild(This, (*parent_entry).data.rightChild);
        }

        if (*parent_entry).data.dirRootEntry != DIRENTRY_NULL {
            (*(*This).entries.offset((*parent_entry).data.dirRootEntry as isize)).parent = parent;
            return TransactedSnapshotImpl_FindFirstChild(This, (*parent_entry).data.dirRootEntry);
        }
    }

    return parent;
}

unsafe fn StorageBaseImpl_IsStreamOpen(stg: *mut StorageBaseImpl, streamEntry: DirRef) -> BOOL {
    let mut strm: *mut StgStreamImpl;

    TRACE!(b"%p,%d\n\0".as_ptr() as PCHAR);

    let head = core::ptr::addr_of_mut!((*stg).strmHead);
    let mut current = (*head).next;
    
    while current != head {
        strm = CONTAINING_RECORD!(current, StgStreamImpl, StrmListEntry);
        
        if (*strm).dirEntry == streamEntry {
            return 1;
        }
        
        current = (*current).next;
    }

    return 0;
}

/// Every offset and HRESULT here is NAMED. The translation spelled them as bare numbers
/// and got four wrong: bigBlockDepotCount was read from 40 (OFFSET_DIRSECTORCOUNT, always
/// 0 in a v3 file) instead of 44 (OFFSET_BBDEPOTCOUNT), so every reopened 512-byte-sector
/// docfile had an empty depot and its first chain walk failed STG_E_READFAULT; and the
/// old-magic / bad-magic / bad-geometry returns were 0x80030001 / 0x80030002 / 0x80030002
/// for STG_E_OLDFORMAT / STG_E_INVALIDHEADER / STG_E_INVALIDHEADER.
unsafe fn StorageImpl_LoadFileHeader(This: *mut StorageImpl) -> HRESULT {
    let mut hr: HRESULT;
    let mut headerBigBlock: [BYTE; HEADER_SIZE as usize] = [0; HEADER_SIZE as usize];
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut bytes_read: DWORD = 0;

    TRACE!(b"\n\0".as_ptr() as *const c_char);

    /* Get a pointer to the big block of data containing the header. */
    offset.u.HighPart = 0;
    offset.u.LowPart = 0;
    hr = StorageImpl_ReadAt(
        This,
        offset,
        headerBigBlock.as_mut_ptr() as *mut core::ffi::c_void,
        HEADER_SIZE,
        &mut bytes_read,
    );
    if SUCCEEDED(hr) && bytes_read != HEADER_SIZE {
        hr = STG_E_FILENOTFOUND;
    }

    /* Extract the information from the header. */
    if SUCCEEDED(hr) {
        /* Check for the "magic number" signature and return an error if it is not found. */
        if memcmp(
            headerBigBlock.as_ptr() as *const core::ffi::c_void,
            STORAGE_oldmagic.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&STORAGE_oldmagic),
        ) == 0
        {
            return STG_E_OLDFORMAT;
        }

        if memcmp(
            headerBigBlock.as_ptr() as *const core::ffi::c_void,
            STORAGE_magic.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&STORAGE_magic),
        ) != 0
        {
            return STG_E_INVALIDHEADER;
        }

        let h = headerBigBlock.as_ptr();
        StorageUtl_ReadWord(h, OFFSET_BIGBLOCKSIZEBITS, &mut (*This).bigBlockSizeBits);
        StorageUtl_ReadWord(h, OFFSET_SMALLBLOCKSIZEBITS, &mut (*This).smallBlockSizeBits);
        StorageUtl_ReadDWord(h, OFFSET_BBDEPOTCOUNT, &mut (*This).bigBlockDepotCount);
        StorageUtl_ReadDWord(h, OFFSET_ROOTSTARTBLOCK, &mut (*This).rootStartBlock);
        StorageUtl_ReadDWord(h, OFFSET_TRANSACTIONSIG, &mut (*This).transactionSig);
        StorageUtl_ReadDWord(h, OFFSET_SMALLBLOCKLIMIT, &mut (*This).smallBlockLimit);
        StorageUtl_ReadDWord(h, OFFSET_SBDEPOTSTART, &mut (*This).smallBlockDepotStart);
        StorageUtl_ReadDWord(h, OFFSET_EXTBBDEPOTSTART, &mut (*This).extBigBlockDepotStart);
        StorageUtl_ReadDWord(h, OFFSET_EXTBBDEPOTCOUNT, &mut (*This).extBigBlockDepotCount);

        for index in 0..COUNT_BBDEPOTINHEADER {
            StorageUtl_ReadDWord(
                h,
                OFFSET_BBDEPOTSTART + (core::mem::size_of::<ULONG>() as ULONG * index as ULONG),
                &mut (*This).bigBlockDepotStart[index as usize],
            );
        }

        /* Make the bitwise arithmetic to get the size of the blocks in bytes. */
        (*This).bigBlockSize = 0x000000001u32 << (*This).bigBlockSizeBits as DWORD;
        (*This).smallBlockSize = 0x000000001u32 << (*This).smallBlockSizeBits as DWORD;

        /* Right now, the code is making some assumptions about the size of the blocks,
           just make sure they are what we're expecting. */
        if ((*This).bigBlockSize != MIN_BIG_BLOCK_SIZE as ULONG
            && (*This).bigBlockSize != MAX_BIG_BLOCK_SIZE as ULONG)
            || (*This).smallBlockSize != DEF_SMALL_BLOCK_SIZE as ULONG
            || (*This).smallBlockLimit != LIMIT_TO_USE_SMALL_BLOCK
        {
            FIXME(b"Broken OLE storage file? bigblock=0x%x, smallblock=0x%x, sblimit=0x%x\n\0".as_ptr() as *const c_char);
            hr = STG_E_INVALIDHEADER;
        } else {
            hr = S_OK;
        }
    }

    hr
}

pub unsafe fn StorageUtl_ReadGUID(buffer: *const BYTE, offset: ULONG, value: *mut GUID) {
    StorageUtl_ReadDWord(buffer, offset, core::ptr::addr_of_mut!((*value).Data1));
    StorageUtl_ReadWord(buffer, offset.wrapping_add(4), core::ptr::addr_of_mut!((*value).Data2));
    StorageUtl_ReadWord(buffer, offset.wrapping_add(6), core::ptr::addr_of_mut!((*value).Data3));

    core::ptr::copy_nonoverlapping( // nolint: silent-failure (element type IS u8 — GUID.Data4 is [u8; 8] — so elements == bytes)
        buffer.add(offset.wrapping_add(8) as usize),
        (*value).Data4.as_mut_ptr(),
        core::mem::size_of_val(&(*value).Data4),
    );
}

unsafe fn StorageImpl_SaveFileHeader(This: *mut StorageImpl) {
    let mut headerBigBlock: [BYTE; HEADER_SIZE as usize] = [0; HEADER_SIZE as usize];
    let offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut bytes_written: DWORD = 0;
    let major_version: DWORD;
    let dirsectorcount: DWORD;

    if (*This).bigBlockSizeBits == 0x9 {
        major_version = 3;
    } else if (*This).bigBlockSizeBits == 0xc {
        major_version = 4;
    } else {
        ERR(b"invalid big block shift 0x%x\n\0".as_ptr() as PCHAR);
        major_version = 4;
    }

    // wine-10 builds the header from scratch. The older read-modify-write kept
    // whatever the medium held, so a commit over an ILockBytes whose header bytes had
    // been overwritten never restored the D0 CF 11 E0 signature.
    core::ptr::copy_nonoverlapping(
        STORAGE_magic.as_ptr(),
        headerBigBlock.as_mut_ptr(),
        STORAGE_magic.len(),
    );

    StorageUtl_WriteWord(headerBigBlock.as_mut_ptr(), OFFSET_MINORVERSION, 0x3e);

    StorageUtl_WriteWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_MAJORVERSION,
        major_version as u16,
    );

    StorageUtl_WriteWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_BYTEORDERMARKER,
        (-2i16) as u16,
    );

    StorageUtl_WriteWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_BIGBLOCKSIZEBITS,
        (*This).bigBlockSizeBits,
    );

    StorageUtl_WriteWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_SMALLBLOCKSIZEBITS,
        (*This).smallBlockSizeBits,
    );

    if major_version >= 4 {
        if !(*This).rootBlockChain.is_null() {
            dirsectorcount = BlockChainStream_GetCount((*This).rootBlockChain);
        } else {
            dirsectorcount = 1;
        }
    } else {
        dirsectorcount = 0;
    }

    StorageUtl_WriteDWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_DIRSECTORCOUNT,
        dirsectorcount,
    );

    StorageUtl_WriteDWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_BBDEPOTCOUNT,
        (*This).bigBlockDepotCount,
    );

    StorageUtl_WriteDWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_ROOTSTARTBLOCK,
        (*This).rootStartBlock,
    );

    StorageUtl_WriteDWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_TRANSACTIONSIG,
        (*This).transactionSig,
    );

    StorageUtl_WriteDWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_SMALLBLOCKLIMIT,
        (*This).smallBlockLimit,
    );

    StorageUtl_WriteDWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_SBDEPOTSTART,
        (*This).smallBlockDepotStart,
    );

    StorageUtl_WriteDWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_SBDEPOTCOUNT,
        if !(*This).smallBlockDepotChain.is_null() {
            BlockChainStream_GetCount((*This).smallBlockDepotChain)
        } else {
            0
        },
    );

    StorageUtl_WriteDWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_EXTBBDEPOTSTART,
        (*This).extBigBlockDepotStart,
    );

    StorageUtl_WriteDWord(
        headerBigBlock.as_mut_ptr(),
        OFFSET_EXTBBDEPOTCOUNT,
        (*This).extBigBlockDepotCount,
    );

    for index in 0..COUNT_BBDEPOTINHEADER {
        StorageUtl_WriteDWord(
            headerBigBlock.as_mut_ptr(),
            OFFSET_BBDEPOTSTART + (core::mem::size_of::<ULONG>() as ULONG * index as ULONG),
            (*This).bigBlockDepotStart[index as usize],
        );
    }

    StorageImpl_WriteAt(
        This,
        offset,
        headerBigBlock.as_ptr() as *const core::ffi::c_void,
        HEADER_SIZE,
        &mut bytes_written,
    );
}

pub unsafe fn StorageUtl_WriteGUID(buffer: *mut BYTE, offset: ULONG, value: *const GUID) {
    StorageUtl_WriteDWord(buffer, offset, (*value).Data1);
    StorageUtl_WriteWord(buffer, offset + 4, (*value).Data2);
    StorageUtl_WriteWord(buffer, offset + 6, (*value).Data3);

    core::ptr::copy_nonoverlapping( // nolint: silent-failure (element type IS u8 — GUID.Data4 is [u8; 8] — so elements == bytes)
        (*value).Data4.as_ptr(),
        buffer.add(offset as usize + 8),
        core::mem::size_of_val(&(*value).Data4)
    );
}

stdmethod! {
fn StorageBaseImpl_Stat(
    iface: *mut IStorage,
    pstatstg: *mut STATSTG,
    grfStatFlag: u32,
) -> HRESULT {
    let This = impl_from_IStorage(iface);
    let mut currentEntry: DirEntry = core::mem::zeroed();
    let res: HRESULT;

    TRACE!(b"(%p, %p, %x)\n\0".as_ptr() as *const i8);

    if pstatstg.is_null() {
        res = E_INVALIDARG;
        'end: loop {
            if res == S_OK {
                TRACE!(b"<-- STATSTG: pwcsName: %s, type: %d, cbSize.Low/High: %d/%d, grfMode: %08x, grfLocksSupported: %d, grfStateBits: %08x\n\0".as_ptr() as *const i8);
            }
            TRACE!(b"<-- %08x\n\0".as_ptr() as *const i8);
            return res;
        }
    }

    if (*This).reverted != 0 {
        res = STG_E_REVERTED;
        'end: loop {
            if res == S_OK {
                TRACE!(b"<-- STATSTG: pwcsName: %s, type: %d, cbSize.Low/High: %d/%d, grfMode: %08x, grfLocksSupported: %d, grfStateBits: %08x\n\0".as_ptr() as *const i8);
            }
            TRACE!(b"<-- %08x\n\0".as_ptr() as *const i8);
            return res;
        }
    }

    res = StorageBaseImpl_ReadDirEntry(This, (*This).storageDirEntry, &mut currentEntry);

    if SUCCEEDED(res) {
        StorageUtl_CopyDirEntryToSTATSTG(This, pstatstg, &currentEntry, grfStatFlag as i32);

        (*pstatstg).grfMode = (*This).openFlags;
        (*pstatstg).grfStateBits = (*This).stateBits;
    }

    'end: loop {
        if res == S_OK {
            TRACE!(b"<-- STATSTG: pwcsName: %s, type: %d, cbSize.Low/High: %d/%d, grfMode: %08x, grfLocksSupported: %d, grfStateBits: %08x\n\0".as_ptr() as *const i8);
        }
        TRACE!(b"<-- %08x\n\0".as_ptr() as *const i8);
        return res;
    }
}
}

/// OleConvertIStorageToOLESTREAMEx (wine-10: still a FIXME stub upstream).
#[no_mangle]
pub unsafe extern "system" fn OleConvertIStorageToOLESTREAMEx(
    stg: LPSTORAGE,
    cf: CLIPFORMAT,
    width: LONG,
    height: LONG,
    size: DWORD,
    medium: LPSTGMEDIUM,
    olestream: LPOLESTREAM,
) -> HRESULT {
    FIXME(b"%p, %x, %ld, %ld, %ld, %p, %p: stub\n\0".as_ptr() as LPCSTR);

    E_NOTIMPL
}

// C: `BOOL convert`. Was `BOOLEAN`, which reads only the low byte: 0x100 was FALSE.
#[no_mangle]
pub unsafe extern "system" fn SetConvertStg(storage: *mut IStorage, convert: BOOL) -> HRESULT {
    static stream_1oleW: [u16; 5] = [1, 'O' as u16, 'l' as u16, 'e' as u16, 0];
    let flags: DWORD = if convert != 0 { 0x00000004 } else { 0 };
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut header: [DWORD; 2] = [0; 2];
    let mut hr: HRESULT;

    

    TRACE!(b"(%p, %d)\n\0".as_ptr() as *const c_char);

    hr = IStorage_OpenStream(
        storage,
        stream_1oleW.as_ptr(),
        core::ptr::null_mut(),
        0x00000012,
        0,
        &mut stream,
    );
    if FAILED(hr) {
        if hr != 0x80030002u32 as i32 {
            return hr;
        }

        return STORAGE_CreateOleStream(storage, flags);
    }

    hr = IStream_Read(
        stream,
        header.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&header) as ULONG,
        core::ptr::null_mut(),
    );
    if FAILED(hr) {
        IStream_Release(stream);
        return hr;
    }

    if (header[1] ^ flags) & 0x00000004 != 0 {
        let mut pos: LARGE_INTEGER = core::mem::zeroed();
        let updated_flags: DWORD;

        if header[1] & 0x00000004 != 0 {
            updated_flags = header[1] & !0x00000004;
        } else {
            updated_flags = header[1] | 0x00000004;
        }

        pos.QuadPart = core::mem::size_of::<DWORD>() as i64;
        // STREAM_SEEK_SET, as the C: this was 1 (CUR), which after the 8-byte header
        // read wrote the convert flag into DWORD 3 of the \1Ole stream.
        hr = IStream_Seek(stream, pos, crate::consts::STREAM_SEEK_SET, core::ptr::null_mut());
        if FAILED(hr) {
            IStream_Release(stream);
            return hr;
        }

        hr = IStream_Write(
            stream,
            &updated_flags as *const DWORD as *const core::ffi::c_void,
            core::mem::size_of::<DWORD>() as ULONG,
            core::ptr::null_mut(),
        );
    }

    IStream_Release(stream);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleSaveToStream(pPStm: *mut IPersistStream, pStm: *mut IStream) -> HRESULT {
    

    let mut clsid: CLSID = core::mem::zeroed();
    let mut res: HRESULT;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char);

    res = IPersistStream_GetClassID(pPStm, &mut clsid);

    if SUCCEEDED(res) {
        res = WriteClassStm(pStm, &clsid);

        if SUCCEEDED(res) {
            res = IPersistStream_Save(pPStm, pStm, 1);
        }
    }

    TRACE!(b"Finished Save\n\0".as_ptr() as *const c_char);
    res
}

unsafe fn STORAGE_WriteCompObj(
    pstg: *mut IStorage,
    clsid: *mut CLSID,
    lpszUserType: LPCWSTR,
    szClipName: LPCWSTR,
    szProgIDName: LPCWSTR,
) -> HRESULT {
    let mut pstm: *mut IStream = core::ptr::null_mut();
    let mut r: HRESULT;
    static szwStreamName: [u16; 9] = [1, b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'O' as u16, b'b' as u16, b'j' as u16, 0];

    static unknown1: [u8; 12] = [
        0x01, 0x00, 0xFE, 0xFF, 0x03, 0x0A, 0x00, 0x00,
        0xFF, 0xFF, 0xFF, 0xFF
    ];
    static unknown2: [u8; 16] = [
        0xF4, 0x39, 0xB2, 0x71, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    ];

    TRACE!(b"%p %s %s %s %s\n\0".as_ptr() as LPCSTR);

    r = IStorage_CreateStream(
        pstg,
        szwStreamName.as_ptr(),
        STGM_CREATE | STGM_WRITE | STGM_SHARE_EXCLUSIVE,
        0,
        0,
        &mut pstm,
    );
    if FAILED(r) {
        return r;
    }

    r = IStream_Write(pstm, unknown1.as_ptr() as *const core::ffi::c_void, unknown1.len() as u32, core::ptr::null_mut());

    if SUCCEEDED(r) {
        r = WriteClassStm(pstm, clsid);
    }

    if SUCCEEDED(r) {
        r = STREAM_WriteString(pstm, lpszUserType);
    }
    if SUCCEEDED(r) {
        r = STREAM_WriteString(pstm, szClipName);
    }
    if SUCCEEDED(r) {
        r = STREAM_WriteString(pstm, szProgIDName);
    }
    if SUCCEEDED(r) {
        r = IStream_Write(pstm, unknown2.as_ptr() as *const core::ffi::c_void, unknown2.len() as u32, core::ptr::null_mut());
    }

    IStream_Release(pstm);

    r
}

// storage32.h declares this WITHOUT `WINAPI`, so it is CDECL, not stdcall. It is
// DECLSPEC_HIDDEN (not an ole32 export), and ring/clipboard.rs declares it `extern "C"`
// — an `extern "system"` definition emits `_Name@N` on i386 and leaves that decl
// dangling. Caught by scan_cc_mismatch.
#[no_mangle]
pub unsafe extern "C" fn OLECONVERT_CreateCompObjStream(
    pStorage: *mut IStorage,
    strOleTypeName: LPCSTR,
) -> HRESULT {
    

    let mut pStream: *mut IStream = core::ptr::null_mut();
    let hStorageRes: HRESULT;
    let mut hRes: HRESULT = 0; // S_OK
    let mut IStorageCompObj: OLECONVERT_ISTORAGE_COMPOBJ = core::mem::zeroed();
    static wstrStreamName: [u16; 9] = [1, 'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'O' as u16, 'b' as u16, 'j' as u16, 0];
    let mut bufferW: [u16; 256] = [0; 256]; // OLESTREAM_MAX_STR_LEN + 1

    static pCompObjUnknown1: [u8; 12] = [0x01, 0x00, 0xFE, 0xFF, 0x03, 0x0A, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF];
    static pCompObjUnknown2: [u8; 4] = [0xF4, 0x39, 0xB2, 0x71];

    memset(
        &mut IStorageCompObj as *mut _ as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<OLECONVERT_ISTORAGE_COMPOBJ>(),
    );
    memcpy(
        IStorageCompObj.byUnknown1.as_mut_ptr() as *mut core::ffi::c_void,
        pCompObjUnknown1.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of_val(&pCompObjUnknown1),
    );
    memcpy(
        IStorageCompObj.byUnknown2.as_mut_ptr() as *mut core::ffi::c_void,
        pCompObjUnknown2.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of_val(&pCompObjUnknown2),
    );

    hStorageRes = IStorage_CreateStream(
        pStorage,
        wstrStreamName.as_ptr(),
        STGM_WRITE | STGM_SHARE_EXCLUSIVE, // storage32.c:10047 (was 0x1000|0x10 = STGM_CREATE — no write bit)
        0,
        0,
        &mut pStream,
    );

    if hStorageRes == 0 {
        IStorageCompObj.dwOleTypeNameLength = (strlen(strOleTypeName) + 1) as DWORD;
        strcpy(IStorageCompObj.strOleTypeName.as_mut_ptr(), strOleTypeName);

        IStorageCompObj.dwProgIDNameLength = (strlen(strOleTypeName) + 1) as DWORD;
        strcpy(IStorageCompObj.strProgIDName.as_mut_ptr(), strOleTypeName);

        MultiByteToWideChar(
            0, // CP_ACP
            0,
            IStorageCompObj.strProgIDName.as_ptr(),
            -1,
            bufferW.as_mut_ptr(),
            256,
        );
        hRes = CLSIDFromProgID(bufferW.as_ptr(), &mut IStorageCompObj.clsid);

        if hRes == 0 {
            let mut hKey: HKEY = core::ptr::null_mut();
            let hErr: LONG = open_classes_key(
                0x80000000u32 as HKEY, // HKEY_CLASSES_ROOT
                bufferW.as_ptr(),
                MAXIMUM_ALLOWED,
                &mut hKey,
            );
            if hErr == 0 {
                let mut strTemp: [i8; 256] = [0; 256];
                IStorageCompObj.dwCLSIDNameLength = 256;
                let hErr2 = RegQueryValueA(
                    hKey,
                    core::ptr::null(),
                    strTemp.as_mut_ptr(),
                    &mut IStorageCompObj.dwCLSIDNameLength as *mut DWORD as *mut LONG,
                );
                if hErr2 == 0 {
                    strcpy(IStorageCompObj.strCLSIDName.as_mut_ptr(), strTemp.as_ptr());
                }
                RegCloseKey(hKey);
            }
        }

        IStream_Write(
            pStream,
            IStorageCompObj.byUnknown1.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&IStorageCompObj.byUnknown1) as ULONG,
            core::ptr::null_mut(),
        );

        WriteClassStm(pStream, &IStorageCompObj.clsid);

        IStream_Write(
            pStream,
            &IStorageCompObj.dwCLSIDNameLength as *const _ as *const core::ffi::c_void,
            core::mem::size_of_val(&IStorageCompObj.dwCLSIDNameLength) as ULONG,
            core::ptr::null_mut(),
        );
        if IStorageCompObj.dwCLSIDNameLength > 0 {
            IStream_Write(
                pStream,
                IStorageCompObj.strCLSIDName.as_ptr() as *const core::ffi::c_void,
                IStorageCompObj.dwCLSIDNameLength,
                core::ptr::null_mut(),
            );
        }
        IStream_Write(
            pStream,
            &IStorageCompObj.dwOleTypeNameLength as *const _ as *const core::ffi::c_void,
            core::mem::size_of_val(&IStorageCompObj.dwOleTypeNameLength) as ULONG,
            core::ptr::null_mut(),
        );
        if IStorageCompObj.dwOleTypeNameLength > 0 {
            IStream_Write(
                pStream,
                IStorageCompObj.strOleTypeName.as_ptr() as *const core::ffi::c_void,
                IStorageCompObj.dwOleTypeNameLength,
                core::ptr::null_mut(),
            );
        }
        IStream_Write(
            pStream,
            &IStorageCompObj.dwProgIDNameLength as *const _ as *const core::ffi::c_void,
            core::mem::size_of_val(&IStorageCompObj.dwProgIDNameLength) as ULONG,
            core::ptr::null_mut(),
        );
        if IStorageCompObj.dwProgIDNameLength > 0 {
            IStream_Write(
                pStream,
                IStorageCompObj.strProgIDName.as_ptr() as *const core::ffi::c_void,
                IStorageCompObj.dwProgIDNameLength,
                core::ptr::null_mut(),
            );
        }
        hRes = IStream_Write(
            pStream,
            IStorageCompObj.byUnknown2.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&IStorageCompObj.byUnknown2) as ULONG,
            core::ptr::null_mut(),
        );
        IStream_Release(pStream);
    }
    hRes
}

#[no_mangle]
pub unsafe extern "system" fn OleLoadFromStream(
    pStm: *mut IStream,
    iidInterface: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    

    let mut clsid: CLSID = core::mem::zeroed();
    let mut xstm: LPVOID = core::ptr::null_mut();

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char);

    let mut res = ReadClassStm(pStm, &mut clsid);
    if FAILED(res) {
        return res;
    }

    const CLSCTX_INPROC_SERVER: u32 = 0x1;
    res = CoCreateInstance(
        &clsid,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        iidInterface,
        ppvObj,
    );
    if FAILED(res) {
        return res;
    }

    res = IUnknown_QueryInterface(*ppvObj as *mut IUnknown, &IID_IPersistStream, &mut xstm);
    if FAILED(res) {
        IUnknown_Release(*ppvObj as *mut IUnknown);
        return res;
    }

    res = IPersistStream_Load(xstm as *mut IPersistStream, pStm);
    IPersistStream_Release(xstm as *mut IPersistStream);

    res
}

unsafe fn StorageBaseImpl_IsStorageOpen(stg: *mut StorageBaseImpl, storageEntry: DirRef) -> BOOL {
    let mut childstg: *mut StorageInternalImpl;
    
    TRACE!(b"%p,%d\n\0".as_ptr() as PCHAR);
    
    let list_head = core::ptr::addr_of_mut!((*stg).storageHead);
    let mut current = (*list_head).next;
    
    while current != list_head {
        childstg = (current as usize - core::mem::offset_of!(StorageInternalImpl, ParentListEntry)) as *mut StorageInternalImpl;
        
        if (*childstg).base.storageDirEntry == storageEntry {
            return 1;
        }
        
        current = (*current).next;
    }
    
    0
}

#[no_mangle]
pub unsafe extern "system" fn ReadFmtUserTypeStg(
    pstg: *mut IStorage,
    pcf: *mut u16,
    lplpszUserType: *mut LPWSTR,
) -> HRESULT {
    

    let mut r: HRESULT;
    let mut stm: *mut IStream = core::ptr::null_mut();
    static szCompObj: [u16; 9] = [1, 'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'O' as u16, 'b' as u16, 'j' as u16, 0];
    let mut unknown1: [u8; 12] = [0; 12];
    let mut unknown2: [u8; 16] = [0; 16];
    let mut count: u32 = 0;
    let mut szProgIDName: LPWSTR = core::ptr::null_mut();
    let mut szCLSIDName: LPWSTR = core::ptr::null_mut();
    let mut szOleTypeName: LPWSTR = core::ptr::null_mut();
    let mut clsid: CLSID = core::mem::zeroed();

    r = IStorage_OpenStream(
        pstg,
        szCompObj.as_ptr(),
        core::ptr::null_mut(),
        STGM_READ | STGM_SHARE_EXCLUSIVE, // storage32.c:9413 (was 0x10|0x40 — two share modes)
        0,
        &mut stm,
    );
    if FAILED(r) {
        return r;
    }

    r = IStream_Read(stm, unknown1.as_mut_ptr() as *mut core::ffi::c_void, 12, &mut count);
    if FAILED(r) || count != 12 {
        CoTaskMemFree(szCLSIDName as *mut core::ffi::c_void);
        CoTaskMemFree(szOleTypeName as *mut core::ffi::c_void);
        CoTaskMemFree(szProgIDName as *mut core::ffi::c_void);
        IStream_Release(stm);
        return r;
    }

    r = ReadClassStm(stm, &mut clsid);
    if FAILED(r) {
        CoTaskMemFree(szCLSIDName as *mut core::ffi::c_void);
        CoTaskMemFree(szOleTypeName as *mut core::ffi::c_void);
        CoTaskMemFree(szProgIDName as *mut core::ffi::c_void);
        IStream_Release(stm);
        return r;
    }

    r = STREAM_ReadString(stm, &mut szCLSIDName);
    if FAILED(r) {
        CoTaskMemFree(szCLSIDName as *mut core::ffi::c_void);
        CoTaskMemFree(szOleTypeName as *mut core::ffi::c_void);
        CoTaskMemFree(szProgIDName as *mut core::ffi::c_void);
        IStream_Release(stm);
        return r;
    }

    r = STREAM_ReadString(stm, &mut szOleTypeName);
    if FAILED(r) {
        CoTaskMemFree(szCLSIDName as *mut core::ffi::c_void);
        CoTaskMemFree(szOleTypeName as *mut core::ffi::c_void);
        CoTaskMemFree(szProgIDName as *mut core::ffi::c_void);
        IStream_Release(stm);
        return r;
    }

    r = STREAM_ReadString(stm, &mut szProgIDName);
    if FAILED(r) {
        CoTaskMemFree(szCLSIDName as *mut core::ffi::c_void);
        CoTaskMemFree(szOleTypeName as *mut core::ffi::c_void);
        CoTaskMemFree(szProgIDName as *mut core::ffi::c_void);
        IStream_Release(stm);
        return r;
    }

    r = IStream_Read(stm, unknown2.as_mut_ptr() as *mut core::ffi::c_void, 16, &mut count);
    if FAILED(r) || count != 16 {
        CoTaskMemFree(szCLSIDName as *mut core::ffi::c_void);
        CoTaskMemFree(szOleTypeName as *mut core::ffi::c_void);
        CoTaskMemFree(szProgIDName as *mut core::ffi::c_void);
        IStream_Release(stm);
        return r;
    }

    if !pcf.is_null() {
        *pcf = RegisterClipboardFormatW(szOleTypeName) as CLIPFORMAT;
    }

    if !lplpszUserType.is_null() {
        *lplpszUserType = szCLSIDName;
        szCLSIDName = core::ptr::null_mut();
    }

    CoTaskMemFree(szCLSIDName as *mut core::ffi::c_void);
    CoTaskMemFree(szOleTypeName as *mut core::ffi::c_void);
    CoTaskMemFree(szProgIDName as *mut core::ffi::c_void);
    IStream_Release(stm);

    r
}

stdmethod! {
fn StorageBaseImpl_OpenStream(
    iface: *mut IStorage,
    pwcsName: *const OLECHAR,
    reserved1: *mut core::ffi::c_void,
    grfMode: DWORD,
    reserved2: DWORD,
    ppstm: *mut *mut IStream,
) -> HRESULT {
    let This = impl_from_IStorage(iface);
    let newStream: *mut StgStreamImpl;
    let mut currentEntry: DirEntry = core::mem::zeroed();
    let streamEntryRef: DirRef;
    let res: HRESULT;

    TRACE!(b"(%p, %s, %p, %x, %d, %p)\n\0".as_ptr() as PCHAR);

    if pwcsName.is_null() || ppstm.is_null() {
        res = E_INVALIDARG;
        return res;
    }

    *ppstm = core::ptr::null_mut();

    if FAILED(validateSTGM(grfMode)) || STGM_SHARE_MODE(grfMode) != STGM_SHARE_EXCLUSIVE {
        res = STG_E_INVALIDFLAG;
        return res;
    }

    if (grfMode & STGM_DELETEONRELEASE) != 0 || (grfMode & STGM_TRANSACTED) != 0 {
        res = STG_E_INVALIDFUNCTION;
        return res;
    }

    if (*This).reverted != 0 {
        res = STG_E_REVERTED;
        return res;
    }

    if ((*This).openFlags & STGM_TRANSACTED) == 0 {
        if STGM_ACCESS_MODE(grfMode) > STGM_ACCESS_MODE((*This).openFlags) {
            res = STG_E_INVALIDFLAG;
            return res;
        }
    }

    streamEntryRef = findElement(This, (*This).storageDirEntry, pwcsName, &mut currentEntry);

    if streamEntryRef != DIRENTRY_NULL && currentEntry.stgType as DWORD == STGTY_STREAM {
        if StorageBaseImpl_IsStreamOpen(This, streamEntryRef) != 0 {
            res = STG_E_ACCESSDENIED;
            return res;
        }

        newStream = StgStreamImpl_Construct(This, grfMode, streamEntryRef);

        if !newStream.is_null() {
            (*newStream).grfMode = grfMode;
            *ppstm = &mut (*newStream).IStream_iface;

            IStream_AddRef(*ppstm);

            res = S_OK;
            return res;
        }

        res = E_OUTOFMEMORY;
        return res;
    }

    res = STG_E_FILENOTFOUND;

    if res == S_OK {
        TRACE!(b"<-- IStream %p\n\0".as_ptr() as PCHAR);
    }
    TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
    res
}
}

unsafe fn removeFromTree(
    This: *mut StorageBaseImpl,
    parentStorageIndex: DirRef,
    deletedIndex: DirRef,
) -> HRESULT {
    let mut entryToDelete: DirEntry = core::mem::zeroed();
    let mut parentEntry: DirEntry = core::mem::zeroed();
    let mut parentEntryRef: DirRef = 0;
    let mut typeOfRelation: ULONG = 0;
    let mut hr: HRESULT;

    hr = StorageBaseImpl_ReadDirEntry(This, deletedIndex, &mut entryToDelete);

    if hr != S_OK {
        return hr;
    }

    hr = findTreeParent(
        This,
        parentStorageIndex,
        entryToDelete.name.as_ptr(),
        &mut parentEntry,
        &mut parentEntryRef,
        &mut typeOfRelation,
    );

    if hr != S_OK {
        return hr;
    }

    if entryToDelete.leftChild != DIRENTRY_NULL {
        setEntryLink(&mut parentEntry, typeOfRelation, entryToDelete.leftChild);

        hr = StorageBaseImpl_WriteDirEntry(This, parentEntryRef, &mut parentEntry);
        if FAILED(hr) {
            return hr;
        }

        if entryToDelete.rightChild != DIRENTRY_NULL {
            let mut newRightChildParent: DirRef = entryToDelete.leftChild;
            let mut newRightChildParentEntry: DirEntry = core::mem::zeroed();

            loop {
                hr = StorageBaseImpl_ReadDirEntry(
                    This,
                    newRightChildParent,
                    &mut newRightChildParentEntry,
                );
                if FAILED(hr) {
                    return hr;
                }

                if newRightChildParentEntry.rightChild != DIRENTRY_NULL {
                    newRightChildParent = newRightChildParentEntry.rightChild;
                } else {
                    break;
                }
            }

            newRightChildParentEntry.rightChild = entryToDelete.rightChild;

            hr = StorageBaseImpl_WriteDirEntry(
                This,
                newRightChildParent,
                &mut newRightChildParentEntry,
            );
            if FAILED(hr) {
                return hr;
            }
        }
    } else {
        setEntryLink(&mut parentEntry, typeOfRelation, entryToDelete.rightChild);

        hr = StorageBaseImpl_WriteDirEntry(This, parentEntryRef, &mut parentEntry);
        if FAILED(hr) {
            return hr;
        }
    }

    return hr;
}

stdmethod! {
fn StorageBaseImpl_CreateStream(
    iface: *mut IStorage,
    pwcsName: *const u16,
    grfMode: u32,
    reserved1: u32,
    reserved2: u32,
    ppstm: *mut *mut IStream,
) -> HRESULT {
    let This = impl_from_IStorage(iface);
    let newStream: *mut StgStreamImpl;
    let mut currentEntry: DirEntry = core::mem::zeroed();
    let mut newStreamEntry: DirEntry = core::mem::zeroed();
    let currentEntryRef: DirRef;
    let mut newStreamEntryRef: DirRef = 0;
    let mut hr: HRESULT;

    

    TRACE!(b"(%p, %s, %x, %d, %d, %p)\n\0".as_ptr() as *const c_char);

    if ppstm.is_null() {
        return 0x80030009u32 as HRESULT;
    }

    if pwcsName.is_null() {
        return STG_E_INVALIDNAME; // C: STG_E_INVALIDNAME (was 0x80030006, STG_E_INVALIDHANDLE)
    }

    if reserved1 != 0 || reserved2 != 0 {
        return STG_E_INVALIDPARAMETER; // C: STG_E_INVALIDPARAMETER (was E_INVALIDARG)
    }

    if FAILED(validateSTGM(grfMode)) {
        return 0x800300FFu32 as HRESULT;
    }

    if STGM_SHARE_MODE(grfMode) != STGM_SHARE_EXCLUSIVE {
        return 0x800300FFu32 as HRESULT;
    }

    if (*This).reverted != 0 {
        return 0x80030102u32 as HRESULT;
    }

    // C: STGM_DELETEONRELEASE (0x04000000) | STGM_TRANSACTED -> STG_E_INVALIDFUNCTION. The
    // translation tested 0x00004000 (no STGM flag at all) and returned STG_E_INVALIDFLAG.
    if (grfMode & STGM_DELETEONRELEASE) != 0 || (grfMode & STGM_TRANSACTED) != 0 {
        return STG_E_INVALIDFUNCTION;
    }

    if ((*This).openFlags & 0x00010000) == 0 {
        if STGM_ACCESS_MODE((*This).openFlags) == 0x00000000 {
            return 0x80030005u32 as HRESULT;
        }

        if STGM_ACCESS_MODE(grfMode) > STGM_ACCESS_MODE((*This).openFlags) {
            return 0x80030005u32 as HRESULT;
        }
    }

    if ((*This).openFlags & 0x08000000) != 0 {
        if (grfMode & STGM_CREATE) != 0 {
            return 0x800300FFu32 as HRESULT;
        }
    }

    *ppstm = core::ptr::null_mut();

    currentEntryRef = findElement(
        This,
        (*This).storageDirEntry,
        pwcsName,
        &mut currentEntry,
    );

    if currentEntryRef != 0xFFFFFFFF {
        if STGM_CREATE_MODE(grfMode) == STGM_CREATE {
            IStorage_DestroyElement(iface, pwcsName);
        } else {
            return 0x80030050u32 as HRESULT;
        }
    }

    memset(
        &mut newStreamEntry as *mut DirEntry as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<DirEntry>(),
    );

    newStreamEntry.sizeOfNameString = ((lstrlenW(pwcsName) + 1) * 2) as WORD;

    if newStreamEntry.sizeOfNameString > DIRENTRY_NAME_BUFFER_LEN as WORD {
        return STG_E_INVALIDNAME; // C: STG_E_INVALIDNAME (was 0x80030006)
    }

    lstrcpyW(newStreamEntry.name.as_mut_ptr(), pwcsName);

    newStreamEntry.stgType = 2;
    newStreamEntry.startingBlock = 0xFFFFFFFE;
    newStreamEntry.size.u.LowPart = 0;
    newStreamEntry.size.u.HighPart = 0;

    newStreamEntry.leftChild = 0xFFFFFFFF;
    newStreamEntry.rightChild = 0xFFFFFFFF;
    newStreamEntry.dirRootEntry = 0xFFFFFFFF;

    hr = StorageBaseImpl_CreateDirEntry(This, &newStreamEntry, &mut newStreamEntryRef);
    if FAILED(hr) {
        return hr;
    }

    hr = insertIntoTree(This, (*This).storageDirEntry, newStreamEntryRef);
    if FAILED(hr) {
        StorageBaseImpl_DestroyDirEntry(This, newStreamEntryRef);
        return hr;
    }

    newStream = StgStreamImpl_Construct(This, grfMode, newStreamEntryRef);

    if !newStream.is_null() {
        *ppstm = &mut (*newStream).IStream_iface;
        IStream_AddRef(*ppstm);
    } else {
        return 0x80030008u32 as HRESULT;
    }

    StorageBaseImpl_Flush(This)
}
}

stdmethod! {
fn StorageBaseImpl_CreateStorage(
    iface: *mut IStorage,
    pwcsName: *const u16,
    grfMode: u32,
    reserved1: u32,
    reserved2: u32,
    ppstg: *mut *mut IStorage,
) -> HRESULT {
    let This = impl_from_IStorage(iface);

    let mut currentEntry: DirEntry = core::mem::zeroed();
    let mut newEntry: DirEntry = core::mem::zeroed();
    let currentEntryRef: DirRef;
    let mut newEntryRef: DirRef = DIRENTRY_NULL;
    let mut hr: HRESULT;

    TRACE!(b"(%p, %s, %x, %d, %d, %p)\n\0".as_ptr() as PCHAR);

    if ppstg.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    if ((*This).openFlags & STGM_SIMPLE) != 0 {
        return STG_E_INVALIDFUNCTION;
    }

    if pwcsName.is_null() {
        return STG_E_INVALIDNAME;
    }

    *ppstg = core::ptr::null_mut();

    if FAILED(validateSTGM(grfMode)) || (grfMode & STGM_DELETEONRELEASE) != 0 {
        WARN!(b"bad grfMode: 0x%x\n\0".as_ptr() as PCHAR);
        return STG_E_INVALIDFLAG;
    }

    if (*This).reverted != 0 {
        return STG_E_REVERTED;
    }

    if ((*This).openFlags & STGM_TRANSACTED) == 0
        && STGM_ACCESS_MODE(grfMode) > STGM_ACCESS_MODE((*This).openFlags)
    {
        WARN!(b"access denied\n\0".as_ptr() as PCHAR);
        return STG_E_ACCESSDENIED;
    }

    currentEntryRef = findElement(This, (*This).storageDirEntry, pwcsName, &mut currentEntry);

    if currentEntryRef != DIRENTRY_NULL {
        if STGM_CREATE_MODE(grfMode) == STGM_CREATE
            && (((*This).openFlags & STGM_TRANSACTED) != 0
                || STGM_ACCESS_MODE((*This).openFlags) != STGM_READ)
        {
            hr = IStorage_DestroyElement(iface, pwcsName);
            if FAILED(hr) {
                return hr;
            }
        } else {
            WARN!(b"file already exists\n\0".as_ptr() as PCHAR);
            return STG_E_FILEALREADYEXISTS;
        }
    } else if ((*This).openFlags & STGM_TRANSACTED) == 0
        && STGM_ACCESS_MODE((*This).openFlags) == STGM_READ
    {
        WARN!(b"read-only storage\n\0".as_ptr() as PCHAR);
        return STG_E_ACCESSDENIED;
    }

    memset(&mut newEntry as *mut _ as *mut _, 0, core::mem::size_of::<DirEntry>());

    newEntry.sizeOfNameString =
        ((lstrlenW(pwcsName) + 1) as usize * core::mem::size_of::<u16>()) as WORD;

    if newEntry.sizeOfNameString as usize > DIRENTRY_NAME_BUFFER_LEN {
        FIXME(b"name too long\n\0".as_ptr() as PCHAR);
        return STG_E_INVALIDNAME;
    }

    lstrcpyW(newEntry.name.as_mut_ptr(), pwcsName);

    newEntry.stgType = STGTY_STORAGE as BYTE;
    newEntry.startingBlock = BLOCK_END_OF_CHAIN;
    newEntry.size.u.LowPart = 0;
    newEntry.size.u.HighPart = 0;

    newEntry.leftChild = DIRENTRY_NULL;
    newEntry.rightChild = DIRENTRY_NULL;
    newEntry.dirRootEntry = DIRENTRY_NULL;

    hr = StorageBaseImpl_CreateDirEntry(This, &newEntry, &mut newEntryRef as *mut _);
    if FAILED(hr) {
        return hr;
    }

    hr = insertIntoTree(This, (*This).storageDirEntry, newEntryRef);
    if FAILED(hr) {
        StorageBaseImpl_DestroyDirEntry(This, newEntryRef);
        return hr;
    }

    hr = IStorage_OpenStorage(
        iface,
        pwcsName,
        core::ptr::null_mut(),
        grfMode,
        core::ptr::null_mut(),
        0,
        ppstg,
    );

    if hr != S_OK || (*ppstg).is_null() {
        return hr;
    }

    if SUCCEEDED(hr) {
        StorageBaseImpl_Flush(This);
    }

    S_OK
}
}

stdmethod! {
fn IEnumSTATSTGImpl_Next(
    iface: *mut IEnumSTATSTG,
    celt: ULONG,
    rgelt: *mut STATSTG,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let This: *mut IEnumSTATSTGImpl = impl_from_IEnumSTATSTG(iface);

    let mut currentEntry: DirEntry = core::mem::zeroed();
    let mut currentReturnStruct: *mut STATSTG = rgelt;
    let mut objectFetched: ULONG = 0;
    let mut currentSearchNode: DirRef = 0;
    let mut hr: HRESULT = S_OK;

    TRACE!(b"%p,%u,%p,%p\n\0".as_ptr() as PCHAR);

    if rgelt.is_null() || (celt != 1 && pceltFetched.is_null()) {
        return E_INVALIDARG;
    }

    if (*(*This).parentStorage).reverted != 0 {
        TRACE!(b"<-- STG_E_REVERTED\n\0".as_ptr() as PCHAR);
        return STG_E_REVERTED;
    }

    let pceltFetched = if pceltFetched.is_null() {
        &mut objectFetched as *mut ULONG
    } else {
        pceltFetched
    };

    *pceltFetched = 0;

    while *pceltFetched < celt {
        hr = IEnumSTATSTGImpl_GetNextRef(This, &mut currentSearchNode);

        if FAILED(hr) || currentSearchNode == DIRENTRY_NULL {
            memset(
                currentReturnStruct as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<STATSTG>(),
            );
            break;
        }

        hr = StorageBaseImpl_ReadDirEntry(
            (*This).parentStorage,
            currentSearchNode,
            &mut currentEntry,
        );
        if FAILED(hr) {
            break;
        }

        StorageUtl_CopyDirEntryToSTATSTG(
            (*This).parentStorage,
            currentReturnStruct,
            &currentEntry,
            STATFLAG_DEFAULT as c_int,
        );

        *pceltFetched += 1;
        currentReturnStruct = currentReturnStruct.add(1);
    }

    if SUCCEEDED(hr) && *pceltFetched != celt {
        hr = S_FALSE;
    }

    TRACE!(b"<-- %08x (asked %u, got %u)\n\0".as_ptr() as PCHAR);
    hr
}
}

stdmethod! {
fn IEnumSTATSTGImpl_Skip(
    iface: *mut IEnumSTATSTG,
    celt: ULONG) -> HRESULT
{
    let This: *mut IEnumSTATSTGImpl = impl_from_IEnumSTATSTG(iface);

    let mut objectFetched: ULONG = 0;
    let mut currentSearchNode: DirRef = 0;
    let mut hr: HRESULT = S_OK;

    TRACE!(b"%p,%u\n\0".as_ptr() as PCHAR);

    if (*(*This).parentStorage).reverted != 0 {
        TRACE!(b"<-- STG_E_REVERTED\n\0".as_ptr() as PCHAR);
        return STG_E_REVERTED;
    }

    while objectFetched < celt {
        hr = IEnumSTATSTGImpl_GetNextRef(This, &mut currentSearchNode);

        if FAILED(hr) || currentSearchNode == DIRENTRY_NULL {
            break;
        }

        objectFetched += 1;
    }

    if SUCCEEDED(hr) && objectFetched != celt {
        return S_FALSE;
    }

    TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
    return hr;
}
}

unsafe extern "C" fn StorageInternalImpl_Destroy(iface: *mut StorageBaseImpl) {
    let This = iface as *mut StorageInternalImpl;

    StorageInternalImpl_Invalidate(&mut (*This).base);

    HeapFree(GetProcessHeap(), 0, This as LPVOID);
}

unsafe extern "C" fn TransactedSharedImpl_Destroy(iface: *mut StorageBaseImpl) {
    let This = iface as *mut TransactedSharedImpl;

    TransactedSharedImpl_Invalidate(core::ptr::addr_of_mut!((*This).base));
    IStorage_Release(core::ptr::addr_of_mut!((*(*This).transactedParent).IStorage_iface));
    IStorage_Release(core::ptr::addr_of_mut!((*(*This).scratch).base.IStorage_iface));
    HeapFree(GetProcessHeap(), 0, This as LPVOID);
}

unsafe fn StorageBaseImpl_CopyStorageTree(
    dst: *mut StorageBaseImpl,
    dst_entry: DirRef,
    src: *mut StorageBaseImpl,
    src_entry: DirRef,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut src_data: DirEntry = core::mem::zeroed();
    let mut dst_data: DirEntry = core::mem::zeroed();
    let mut new_root_entry: DirRef = 0;

    hr = StorageBaseImpl_ReadDirEntry(src, src_entry, &mut src_data);

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_DupStorageTree(dst, &mut new_root_entry, src, src_data.dirRootEntry);
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_ReadDirEntry(dst, dst_entry, &mut dst_data);
        dst_data.clsid = src_data.clsid;
        dst_data.ctime = src_data.ctime;
        dst_data.mtime = src_data.mtime;
        dst_data.dirRootEntry = new_root_entry;
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_WriteDirEntry(dst, dst_entry, &dst_data);
    }

    hr
}

stdmethod! {
fn StorageBaseImpl_MoveElementTo(
    iface: *mut IStorage,
    pwcsName: *const OLECHAR,
    pstgDest: *mut IStorage,
    pwcsNewName: *const OLECHAR,
    grfFlags: DWORD,
) -> HRESULT {
    
    
    FIXME(b"(%p %s %p %s %u): stub\n\0".as_ptr() as *const c_char);
    E_NOTIMPL
}
}

stdmethod! {
fn TransactedSharedImpl_Commit(
    iface: *mut IStorage,
    grfCommitFlags: DWORD,
) -> HRESULT {
    let This = impl_from_IStorage(iface) as *mut TransactedSharedImpl;
    let mut new_storage_ref: DirRef = 0;
    let mut prev_storage_ref: DirRef = DIRENTRY_NULL;
    let mut src_data: DirEntry = core::mem::zeroed();
    let mut dst_data: DirEntry = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut transactionSig: ULONG = 0;

    TRACE!(b"(%p,%x)\n\0".as_ptr() as PCHAR);

    if STGM_ACCESS_MODE((*This).base.openFlags) == STGM_READ {
        return STG_E_ACCESSDENIED;
    }

    hr = StorageBaseImpl_LockTransaction((*This).transactedParent, 1);
    if hr == E_NOTIMPL {
        hr = S_OK;
    }
    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_GetTransactionSig((*This).transactedParent, &mut transactionSig, 1);
        if SUCCEEDED(hr) {
            if (grfCommitFlags & STGC_ONLYIFCURRENT) != 0 && transactionSig != (*This).lastTransactionSig {
                hr = STG_E_NOTCURRENT;
            }

            if SUCCEEDED(hr) {
                hr = StorageBaseImpl_SetTransactionSig((*This).transactedParent, transactionSig + 1);
            }
        } else if hr == E_NOTIMPL {
            hr = S_OK;
        }

        if SUCCEEDED(hr) {
            hr = StorageBaseImpl_ReadDirEntry(&mut (*(*This).scratch).base, (*(*This).scratch).base.storageDirEntry, &mut src_data);
        }

        if SUCCEEDED(hr) {
            hr = StorageBaseImpl_DupStorageTree((*This).transactedParent, &mut new_storage_ref, &mut (*(*This).scratch).base, src_data.dirRootEntry);
        }

        if SUCCEEDED(hr) {
            hr = StorageBaseImpl_Flush((*This).transactedParent);
        }

        if SUCCEEDED(hr) {
            hr = StorageBaseImpl_ReadDirEntry((*This).transactedParent, (*(*This).transactedParent).storageDirEntry, &mut dst_data);
        }

        if SUCCEEDED(hr) {
            prev_storage_ref = dst_data.dirRootEntry;
            dst_data.dirRootEntry = new_storage_ref;
            dst_data.clsid = src_data.clsid;
            dst_data.ctime = src_data.ctime;
            dst_data.mtime = src_data.mtime;
            hr = StorageBaseImpl_WriteDirEntry((*This).transactedParent, (*(*This).transactedParent).storageDirEntry, &mut dst_data);
        }

        if SUCCEEDED(hr) {
            StorageBaseImpl_Flush((*This).transactedParent);

            hr = StorageBaseImpl_DeleteStorageTree((*This).transactedParent, prev_storage_ref, 1);
        }

        if SUCCEEDED(hr) {
            hr = StorageBaseImpl_Flush((*This).transactedParent);
        }

        StorageBaseImpl_UnlockTransaction((*This).transactedParent, 1);

        if SUCCEEDED(hr) {
            hr = IStorage_Commit(&mut (*(*This).scratch).base.IStorage_iface, STGC_DEFAULT);
        }

        if SUCCEEDED(hr) {
            (*This).lastTransactionSig = transactionSig + 1;
        }
    }
    TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
    return hr;
}
}

unsafe fn Storage32Impl_GetExtDepotBlock(This: *mut StorageImpl, depotIndex: ULONG) -> ULONG {
    let depotBlocksPerExtBlock: ULONG = ((*This).bigBlockSize / core::mem::size_of::<ULONG>() as u32) - 1;
    let numExtBlocks: ULONG = depotIndex - COUNT_BBDEPOTINHEADER as ULONG;
    let extBlockCount: ULONG = numExtBlocks / depotBlocksPerExtBlock;
    let extBlockOffset: ULONG = numExtBlocks % depotBlocksPerExtBlock;
    let mut blockIndex: ULONG = BLOCK_UNUSED;
    let extBlockIndex: ULONG;
    let mut depotBuffer: [BYTE; MAX_BIG_BLOCK_SIZE as usize] = [0; MAX_BIG_BLOCK_SIZE as usize];
    let _index: i32;
    let num_blocks: i32;

    assert(depotIndex >= COUNT_BBDEPOTINHEADER as ULONG);

    if extBlockCount >= (*This).extBigBlockDepotCount {
        return BLOCK_UNUSED;
    }

    if (*This).indexExtBlockDepotCached != extBlockCount {
        extBlockIndex = *(*This).extBigBlockDepotLocations.offset((extBlockCount as usize) as isize);

        StorageImpl_ReadBigBlock(This, extBlockIndex, depotBuffer.as_mut_ptr() as *mut core::ffi::c_void, core::ptr::null_mut());

        num_blocks = ((*This).bigBlockSize / 4) as i32;

        for index in 0..num_blocks {
            StorageUtl_ReadDWord(depotBuffer.as_ptr(), (index as ULONG) * core::mem::size_of::<ULONG>() as u32, core::ptr::addr_of_mut!(blockIndex));
            (*This).extBlockDepotCached[index as usize] = blockIndex;
        }

        (*This).indexExtBlockDepotCached = extBlockCount;
    }

    blockIndex = (*This).extBlockDepotCached[extBlockOffset as usize];

    blockIndex
}

unsafe fn Storage32Impl_GetNextExtendedBlock(This: *mut StorageImpl, blockIndex: ULONG) -> ULONG {
    let mut nextBlockIndex: ULONG = BLOCK_SPECIAL;
    let depotBlockOffset: ULONG = (*This).bigBlockSize - core::mem::size_of::<ULONG>() as ULONG;

    StorageImpl_ReadDWordFromBigBlock(This, blockIndex, depotBlockOffset, &mut nextBlockIndex);

    nextBlockIndex
}

unsafe fn Storage32Impl_AddBlockDepot(This: *mut StorageImpl, blockIndex: ULONG, depotIndex: ULONG)
{
  let mut blockBuffer: [BYTE; MAX_BIG_BLOCK_SIZE as usize] = [0; MAX_BIG_BLOCK_SIZE as usize];
  let rangeLockIndex: ULONG = RANGELOCK_FIRST / (*This).bigBlockSize - 1;
  let blocksPerDepot: ULONG = (*This).bigBlockSize / core::mem::size_of::<ULONG>() as ULONG;
  let rangeLockDepot: ULONG = rangeLockIndex / blocksPerDepot;

  core::ptr::write_bytes(blockBuffer.as_mut_ptr(), BLOCK_UNUSED as u8, (*This).bigBlockSize as usize);

  if depotIndex == rangeLockDepot
  {
    let offset = (rangeLockIndex % blocksPerDepot) as usize;
    let ptr = blockBuffer.as_mut_ptr() as *mut ULONG;
    ptr.add(offset).write_unaligned(BLOCK_END_OF_CHAIN);
  }

  StorageImpl_WriteBigBlock(This, blockIndex, blockBuffer.as_ptr() as *const core::ffi::c_void);
}

unsafe fn BlockChainStream_GetBlockAtOffset(
    This: *mut BlockChainStream,
    index: ULONG,
    block: *mut *mut BlockChainBlock,
    sector: *mut ULONG,
    create: BOOL,
) -> HRESULT {
    let mut result: *mut BlockChainBlock = core::ptr::null_mut();
    let mut i: i32;

    i = 0;
    while i < 2 {
        if (*This).cachedBlocks[i as usize].index == index {
            *sector = (*This).cachedBlocks[i as usize].sector;
            *block = core::ptr::addr_of_mut!((*This).cachedBlocks[i as usize]);
            return 0x00000000; // S_OK
        }
        i += 1;
    }

    *sector = BlockChainStream_GetSectorOfOffset(This, index);
    if *sector == BLOCK_END_OF_CHAIN {
        // The literal here was 0x80030006 = STG_E_INVALIDHANDLE, despite the comment
        // claiming STG_E_DOCFILECORRUPT (0x80030109). Named constant, per the C.
        return STG_E_DOCFILECORRUPT;
    }

    if create != 0 {
        if (*This).cachedBlocks[0].index == 0xffffffff {
            result = core::ptr::addr_of_mut!((*This).cachedBlocks[0]);
        } else if (*This).cachedBlocks[1].index == 0xffffffff {
            result = core::ptr::addr_of_mut!((*This).cachedBlocks[1]);
        } else {
            result = core::ptr::addr_of_mut!((*This).cachedBlocks[(*This).blockToEvict as usize]);
            (*This).blockToEvict += 1;
            if (*This).blockToEvict == 2 {
                (*This).blockToEvict = 0;
            }
        }

        if (*result).dirty != 0 {
            if StorageImpl_WriteBigBlock(
                (*This).parentStorage,
                (*result).sector,
                (*result).data.as_ptr() as *const core::ffi::c_void,
            ) == 0
            {
                return STG_E_WRITEFAULT;
            }
            (*result).dirty = 0;
        }

        (*result).read = 0;
        (*result).index = index;
        (*result).sector = *sector;
    }

    *block = result;
    0x00000000 // S_OK
}

unsafe fn BlockChainStream_Flush(This: *mut BlockChainStream) -> HRESULT {
    let mut i: i32;
    if This.is_null() {
        return S_OK;
    }
    i = 0;
    while i < 2 {
        if (*This).cachedBlocks[i as usize].dirty != 0 {
            if StorageImpl_WriteBigBlock(
                (*This).parentStorage,
                (*This).cachedBlocks[i as usize].sector,
                (*This).cachedBlocks[i as usize].data.as_ptr() as *const core::ffi::c_void,
            ) != 0
            {
                (*This).cachedBlocks[i as usize].dirty = 0;
            } else {
                return STG_E_WRITEFAULT;
            }
        }
        i += 1;
    }
    S_OK
}

unsafe fn Storage32Impl_SetExtDepotBlock(This: *mut StorageImpl, depotIndex: ULONG, blockIndex: ULONG) {
    let depotBlocksPerExtBlock = ((*This).bigBlockSize / core::mem::size_of::<ULONG>() as ULONG) - 1;
    let numExtBlocks = depotIndex - COUNT_BBDEPOTINHEADER as ULONG;
    let extBlockCount = numExtBlocks / depotBlocksPerExtBlock;
    let extBlockOffset = numExtBlocks % depotBlocksPerExtBlock;
    let extBlockIndex: ULONG;

    
    assert((depotIndex >= COUNT_BBDEPOTINHEADER as ULONG) as i32);

    assert((extBlockCount < (*This).extBigBlockDepotCount) as i32);

    extBlockIndex = *(*This).extBigBlockDepotLocations.offset((extBlockCount as usize) as isize);

    if extBlockIndex != BLOCK_UNUSED {
        StorageImpl_WriteDWordToBigBlock(This, extBlockIndex,
                            extBlockOffset * core::mem::size_of::<ULONG>() as ULONG,
                            blockIndex);
    }

    if (*This).indexExtBlockDepotCached == extBlockCount {
        (*This).extBlockDepotCached[extBlockOffset as usize] = blockIndex;
    }
}

unsafe fn Storage32Impl_AddExtBlockDepot(This: *mut StorageImpl) -> ULONG {
    
    

    let numExtBlocks = (*This).extBigBlockDepotCount;
    let mut nextExtBlock = (*This).extBigBlockDepotStart;
    let mut depotBuffer: [BYTE; MAX_BIG_BLOCK_SIZE as usize] = [0; MAX_BIG_BLOCK_SIZE as usize];
    let nextBlockOffset = (*This).bigBlockSize - core::mem::size_of::<ULONG>() as ULONG;
    let blocksPerDepotBlock = (*This).bigBlockSize / core::mem::size_of::<ULONG>() as ULONG;
    let depotBlocksPerExtBlock = blocksPerDepotBlock - 1;

    let index = (COUNT_BBDEPOTINHEADER as ULONG + (numExtBlocks * depotBlocksPerExtBlock))
        * blocksPerDepotBlock;

    if numExtBlocks == 0 && nextExtBlock == BLOCK_END_OF_CHAIN {
        (*This).extBigBlockDepotStart = index;
    } else {
        nextExtBlock = *(*This).extBigBlockDepotLocations.offset(((*This).extBigBlockDepotCount - 1) as isize);
        StorageImpl_WriteDWordToBigBlock(This, nextExtBlock, nextBlockOffset, index);
    }

    memset(depotBuffer.as_mut_ptr() as *mut core::ffi::c_void, BLOCK_UNUSED as i32, (*This).bigBlockSize as usize);
    StorageImpl_WriteBigBlock(This, index, depotBuffer.as_ptr() as *const core::ffi::c_void);

    if (*This).extBigBlockDepotLocationsSize == numExtBlocks {
        let new_cache_size = ((*This).extBigBlockDepotLocationsSize + 1) * 2;
        let new_cache = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<ULONG>() * new_cache_size as usize) as SIZE_T) as *mut ULONG;

        memcpy(
            new_cache as *mut core::ffi::c_void,
            (*This).extBigBlockDepotLocations as *const core::ffi::c_void,
            core::mem::size_of::<ULONG>() * (*This).extBigBlockDepotLocationsSize as usize
        );
        HeapFree(GetProcessHeap(), 0, (*This).extBigBlockDepotLocations as LPVOID);

        (*This).extBigBlockDepotLocations = new_cache;
        (*This).extBigBlockDepotLocationsSize = new_cache_size;
    }
    *(*This).extBigBlockDepotLocations.offset(numExtBlocks as isize) = index;

    index
}

unsafe extern "C" fn StorageImpl_LockTransaction(base: *mut StorageBaseImpl, write: BOOL) -> HRESULT {
    let This = base as *mut StorageImpl;
    let hr: HRESULT;
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut cb: ULARGE_INTEGER = core::mem::zeroed();

    if write != 0 {
        offset.QuadPart = RANGELOCK_TRANSACTION_FIRST as ULONGLONG;
        cb.QuadPart = (RANGELOCK_TRANSACTION_LAST - RANGELOCK_TRANSACTION_FIRST + 1) as ULONGLONG;
    } else {
        offset.QuadPart = RANGELOCK_COMMIT as ULONGLONG;
        cb.QuadPart = 1;
    }

    hr = StorageImpl_LockRegionSync(This, offset, cb, LOCK_ONLYONCE, core::ptr::null_mut());

    hr
}

unsafe fn StorageImpl_GrabLocks(This: *mut StorageImpl, openFlags: DWORD) -> HRESULT {
    let mut hr: HRESULT;
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut cb: ULARGE_INTEGER = core::mem::zeroed();
    let mut share_mode: DWORD = STGM_SHARE_MODE(openFlags);
    let mut supported: BOOL = 0;

    if (openFlags & STGM_NOSNAPSHOT) != 0 {
        if share_mode == STGM_SHARE_DENY_READ {
            share_mode = STGM_SHARE_EXCLUSIVE;
        } else if share_mode != STGM_SHARE_EXCLUSIVE {
            share_mode = STGM_SHARE_DENY_WRITE;
        }
    }

    // wine-10.0: a read-only, deny-write open takes neither the READ nor the
    // DENY_WRITE lock, so it can coexist with a STGM_PRIORITY opener (Wine's
    // test_storage_refcount opens exactly this while one is held).
    let ro_denyw =
        STGM_ACCESS_MODE(openFlags) == STGM_READ && share_mode == STGM_SHARE_DENY_WRITE;

    offset.QuadPart = RANGELOCK_CHECKLOCKS as ULONGLONG;
    cb.QuadPart = 1;
    hr = StorageImpl_LockRegionSync(This, offset, cb, LOCK_ONLYONCE, &mut supported);

    if supported == 0 {
        return S_OK;
    } else if FAILED(hr) {
        return hr;
    }

    hr = S_OK;

    if (openFlags & STGM_PRIORITY) == STGM_PRIORITY {
        hr = StorageImpl_CheckLockRange(This, RANGELOCK_COMMIT, RANGELOCK_COMMIT, STG_E_LOCKVIOLATION);
    }

    if SUCCEEDED(hr) && (STGM_ACCESS_MODE(openFlags) != STGM_WRITE) {
        hr = StorageImpl_CheckLockRange(This, RANGELOCK_DENY_READ_FIRST, RANGELOCK_DENY_READ_LAST, STG_E_SHAREVIOLATION);
    }

    if SUCCEEDED(hr) && (STGM_ACCESS_MODE(openFlags) != STGM_READ) {
        hr = StorageImpl_CheckLockRange(This, RANGELOCK_DENY_WRITE_FIRST, RANGELOCK_DENY_WRITE_LAST, STG_E_SHAREVIOLATION);
    }

    if SUCCEEDED(hr) && (share_mode == STGM_SHARE_DENY_READ || share_mode == STGM_SHARE_EXCLUSIVE) {
        hr = StorageImpl_CheckLockRange(This, RANGELOCK_READ_FIRST, RANGELOCK_READ_LAST, STG_E_LOCKVIOLATION);
    }

    if SUCCEEDED(hr) && (share_mode == STGM_SHARE_DENY_WRITE || share_mode == STGM_SHARE_EXCLUSIVE) {
        hr = StorageImpl_CheckLockRange(This, RANGELOCK_WRITE_FIRST, RANGELOCK_WRITE_LAST, STG_E_LOCKVIOLATION);
    }

    if SUCCEEDED(hr) && STGM_ACCESS_MODE(openFlags) == STGM_READ && share_mode == STGM_SHARE_EXCLUSIVE {
        hr = StorageImpl_CheckLockRange(This, 0, RANGELOCK_CHECKLOCKS - 1, STG_E_LOCKVIOLATION);

        if SUCCEEDED(hr) {
            hr = StorageImpl_CheckLockRange(This, RANGELOCK_CHECKLOCKS + 1, RANGELOCK_LAST, STG_E_LOCKVIOLATION);
        }
    }

    if SUCCEEDED(hr) && (openFlags & STGM_PRIORITY) == STGM_PRIORITY {
        hr = StorageImpl_LockOne(This, RANGELOCK_PRIORITY1_FIRST, RANGELOCK_PRIORITY1_LAST);
        if SUCCEEDED(hr) {
            hr = StorageImpl_LockOne(This, RANGELOCK_PRIORITY2_FIRST, RANGELOCK_PRIORITY2_LAST);
        }
    }

    if SUCCEEDED(hr) && (STGM_ACCESS_MODE(openFlags) != STGM_WRITE) && !ro_denyw {
        hr = StorageImpl_LockOne(This, RANGELOCK_READ_FIRST, RANGELOCK_READ_LAST);
    }

    if SUCCEEDED(hr) && (STGM_ACCESS_MODE(openFlags) != STGM_READ) {
        hr = StorageImpl_LockOne(This, RANGELOCK_WRITE_FIRST, RANGELOCK_WRITE_LAST);
    }

    if SUCCEEDED(hr) && (share_mode == STGM_SHARE_DENY_READ || share_mode == STGM_SHARE_EXCLUSIVE) {
        hr = StorageImpl_LockOne(This, RANGELOCK_DENY_READ_FIRST, RANGELOCK_DENY_READ_LAST);
    }

    if SUCCEEDED(hr) && (share_mode == STGM_SHARE_DENY_WRITE || share_mode == STGM_SHARE_EXCLUSIVE) && !ro_denyw {
        hr = StorageImpl_LockOne(This, RANGELOCK_DENY_WRITE_FIRST, RANGELOCK_DENY_WRITE_LAST);
    }

    if SUCCEEDED(hr) && (openFlags & STGM_NOSNAPSHOT) == STGM_NOSNAPSHOT {
        hr = StorageImpl_LockOne(This, RANGELOCK_NOSNAPSHOT_FIRST, RANGELOCK_NOSNAPSHOT_LAST);
    }

    offset.QuadPart = RANGELOCK_CHECKLOCKS as ULONGLONG;
    cb.QuadPart = 1;
    StorageImpl_UnlockRegion(This, offset, cb, LOCK_ONLYONCE);

    hr
}

stdmethod! {
fn StorageBaseImpl_Revert(
    iface: *mut IStorage
) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as LPCSTR);
    S_OK
}
}

unsafe fn TransactedSnapshotImpl_EnsureReadEntry(
    This: *mut TransactedSnapshotImpl,
    entry: DirRef,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut data: DirEntry = core::mem::zeroed();

    if (*(*This).entries.offset(entry as isize)).read == 0 {
        hr = StorageBaseImpl_ReadDirEntry(
            (*This).transactedParent,
            (*(*This).entries.offset(entry as isize)).transactedParentEntry,
            &mut data,
        );

        if SUCCEEDED(hr) && data.leftChild != DIRENTRY_NULL {
            data.leftChild = TransactedSnapshotImpl_CreateStubEntry(This, data.leftChild);

            if data.leftChild == DIRENTRY_NULL {
                hr = E_OUTOFMEMORY;
            }
        }

        if SUCCEEDED(hr) && data.rightChild != DIRENTRY_NULL {
            data.rightChild = TransactedSnapshotImpl_CreateStubEntry(This, data.rightChild);

            if data.rightChild == DIRENTRY_NULL {
                hr = E_OUTOFMEMORY;
            }
        }

        if SUCCEEDED(hr) && data.dirRootEntry != DIRENTRY_NULL {
            data.dirRootEntry = TransactedSnapshotImpl_CreateStubEntry(This, data.dirRootEntry);

            if data.dirRootEntry == DIRENTRY_NULL {
                hr = E_OUTOFMEMORY;
            }
        }

        if SUCCEEDED(hr) {
            core::ptr::copy_nonoverlapping(
                &data as *const DirEntry,
                &mut (*(*This).entries.offset(entry as isize)).data as *mut DirEntry,
                1,
            );
            (*(*This).entries.offset(entry as isize)).read = 1;
        }
    }

    hr
}

unsafe fn TransactedSnapshotImpl_MakeStreamDirty(
    This: *mut TransactedSnapshotImpl,
    entry: DirRef,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if (*This).entries.offset(entry as isize).read().stream_dirty == 0 {
        let mut new_entrydata: DirEntry = core::mem::zeroed();
        
        new_entrydata.name[0] = 'S' as u16;
        new_entrydata.sizeOfNameString = 1;
        new_entrydata.stgType = STGTY_STREAM as BYTE;
        new_entrydata.startingBlock = BLOCK_END_OF_CHAIN;
        new_entrydata.leftChild = DIRENTRY_NULL;
        new_entrydata.rightChild = DIRENTRY_NULL;
        new_entrydata.dirRootEntry = DIRENTRY_NULL;

        hr = StorageBaseImpl_CreateDirEntry(
            (*This).scratch,
            &mut new_entrydata,
            core::ptr::addr_of_mut!((*(*This).entries.offset(entry as isize)).stream_entry),
        );

        if SUCCEEDED(hr) && (*(*This).entries.offset(entry as isize)).transactedParentEntry != DIRENTRY_NULL {
            hr = StorageBaseImpl_CopyStream(
                (*This).scratch,
                (*(*This).entries.offset(entry as isize)).stream_entry,
                (*This).transactedParent,
                (*(*This).entries.offset(entry as isize)).transactedParentEntry,
            );

            if FAILED(hr) {
                StorageBaseImpl_DestroyDirEntry(
                    (*This).scratch,
                    (*(*This).entries.offset(entry as isize)).stream_entry,
                );
            }
        }

        if SUCCEEDED(hr) {
            (*(*This).entries.offset(entry as isize)).stream_dirty = 1;
        }

        if (*(*This).entries.offset(entry as isize)).transactedParentEntry != DIRENTRY_NULL {
            let delete_ref: DirRef = TransactedSnapshotImpl_CreateStubEntry(
                This,
                (*(*This).entries.offset(entry as isize)).transactedParentEntry,
            );

            if delete_ref != DIRENTRY_NULL {
                (*(*This).entries.offset(delete_ref as isize)).deleted = 1;
            }

            (*(*This).entries.offset(entry as isize)).transactedParentEntry = DIRENTRY_NULL;
            (*(*This).entries.offset(entry as isize)).newTransactedParentEntry = DIRENTRY_NULL;
        }
    }

    hr
}

stdmethod! {
fn TransactedSnapshotImpl_Revert(
    iface: *mut IStorage
) -> HRESULT {
    let This = impl_from_IStorage(iface) as *mut TransactedSnapshotImpl;
    let mut zero: ULARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG;

    TRACE!(b"(%p)\n\0".as_ptr() as LPCSTR);

    StorageBaseImpl_DeleteAll(&mut (*This).base);

    zero.QuadPart = 0;
    i = 0;
    while i < (*This).entries_size {
        if (*(*This).entries.offset(i as isize)).stream_dirty != 0 {
            StorageBaseImpl_StreamSetSize(
                (*This).scratch,
                (*(*This).entries.offset(i as isize)).stream_entry,
                zero
            );

            StorageBaseImpl_DestroyDirEntry(
                (*This).scratch,
                (*(*This).entries.offset(i as isize)).stream_entry
            );
        }
        i += 1;
    }

    core::ptr::write_bytes(
        (*This).entries,
        0,
        (*This).entries_size as usize
    );

    (*This).firstFreeEntry = 0;
    (*This).base.storageDirEntry = TransactedSnapshotImpl_CreateStubEntry(
        This,
        (*(*This).transactedParent).storageDirEntry
    );

    S_OK
}
}

unsafe fn TransactedSnapshotImpl_DestroyTemporaryCopy(
    This: *mut TransactedSnapshotImpl,
    stop: DirRef,
) {
    let mut cursor: DirRef;
    let mut entry: *mut TransactedDirEntry;
    let mut zero: ULARGE_INTEGER = core::mem::zeroed();

    zero.QuadPart = 0;

    if (*(*This).entries.offset((*This).base.storageDirEntry as isize)).read == 0 {
        return;
    }

    cursor = (*(*This).entries.offset((*This).base.storageDirEntry as isize)).data.dirRootEntry;

    if cursor == DIRENTRY_NULL {
        return;
    }

    cursor = TransactedSnapshotImpl_FindFirstChild(This, cursor);

    while cursor != DIRENTRY_NULL && cursor != stop {
        if TransactedSnapshotImpl_MadeCopy(This, cursor) != 0 {
            entry = (*This).entries.offset(cursor as isize);

            if (*entry).stream_dirty != 0 {
                StorageBaseImpl_StreamSetSize(
                    (*This).transactedParent,
                    (*entry).newTransactedParentEntry,
                    zero,
                );
            }

            StorageBaseImpl_DestroyDirEntry(
                (*This).transactedParent,
                (*entry).newTransactedParentEntry,
            );

            (*entry).newTransactedParentEntry = (*entry).transactedParentEntry;
        }

        cursor = TransactedSnapshotImpl_FindNextChild(This, cursor);
    }
}

unsafe extern "C" fn StorageImpl_SetTransactionSig(
    base: *mut StorageBaseImpl,
    value: ULONG,
) -> HRESULT {
    let This = base as *mut StorageImpl;

    (*This).transactionSig = value;
    StorageImpl_SaveFileHeader(This);

    S_OK
}

unsafe fn UpdateRawDirEntry(buffer: *mut BYTE, newData: *const DirEntry) {
    core::ptr::write_bytes(buffer, 0, RAW_DIRENTRY_SIZE as usize);

    core::ptr::copy_nonoverlapping(
        (*newData).name.as_ptr() as *const BYTE,
        buffer.add(OFFSET_PS_NAME as usize),
        DIRENTRY_NAME_BUFFER_LEN,
    );

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*newData).stgType) as *const BYTE,
        buffer.add(OFFSET_PS_STGTYPE as usize),
        1,
    );

    StorageUtl_WriteWord(
        buffer,
        OFFSET_PS_NAMELENGTH,
        (*newData).sizeOfNameString,
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_LEFTCHILD,
        (*newData).leftChild,
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_RIGHTCHILD,
        (*newData).rightChild,
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_DIRROOT,
        (*newData).dirRootEntry,
    );

    StorageUtl_WriteGUID(
        buffer,
        OFFSET_PS_GUID,
        core::ptr::addr_of!((*newData).clsid),
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_CTIMELOW,
        (*newData).ctime.dwLowDateTime,
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_CTIMEHIGH,
        (*newData).ctime.dwHighDateTime,
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_MTIMELOW,
        (*newData).mtime.dwLowDateTime,
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_MTIMEHIGH,
        // wine-10 fix: was ctime's high half.
        (*newData).mtime.dwHighDateTime,
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_STARTBLOCK,
        (*newData).startingBlock,
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_SIZE,
        (*newData).size.u.LowPart,
    );

    StorageUtl_WriteDWord(
        buffer,
        OFFSET_PS_SIZE_HIGH,
        (*newData).size.u.HighPart,
    );
}

#[no_mangle]
pub unsafe extern "system" fn WriteFmtUserTypeStg(
    pstg: *mut IStorage,
    cf: u16,
    lpszUserType: LPOLESTR,
) -> HRESULT {
    let mut stat: STATSTG = core::mem::zeroed();
    let mut r: HRESULT;
    let mut szwClipName: [u16; 0x40] = [0; 0x40];
    let mut clsid: CLSID;
    let mut wstrProgID: LPWSTR = core::ptr::null_mut();
    let n: u32;

    

    const STATFLAG_NONAME: u32 = 1;

    if cf != 0 {
        n = GetClipboardFormatNameW(cf as u32, szwClipName.as_mut_ptr(), 0x40) as u32;
        szwClipName[n as usize] = 0;
    }

    r = IStorage_Stat(pstg, &mut stat, STATFLAG_NONAME);
    if r >= 0 {
        clsid = stat.clsid;
    } else {
        clsid = core::mem::zeroed();
    }

    ProgIDFromCLSID(&clsid, &mut wstrProgID);

    r = STORAGE_WriteCompObj(
        pstg,
        &mut clsid,
        lpszUserType,
        if cf != 0 { szwClipName.as_ptr() } else { core::ptr::null() },
        wstrProgID,
    );

    CoTaskMemFree(wstrProgID as *mut core::ffi::c_void);

    r
}

stdmethod! {
fn StorageBaseImpl_RenameElement(
    iface: *mut IStorage,
    pwcsOldName: *const u16,
    pwcsNewName: *const u16,
) -> HRESULT {
    let This = impl_from_IStorage(iface);
    let mut currentEntry: DirEntry = core::mem::zeroed();
    let mut currentEntryRef: DirRef;

    TRACE!(b"(%p, %s, %s)\n\0".as_ptr() as PCHAR);

    if (*This).reverted != 0 {
        return STG_E_REVERTED;
    }

    currentEntryRef = findElement(
        This,
        (*This).storageDirEntry,
        pwcsNewName,
        &mut currentEntry,
    );

    if currentEntryRef != DIRENTRY_NULL {
        return STG_E_FILEALREADYEXISTS;
    }

    currentEntryRef = findElement(
        This,
        (*This).storageDirEntry,
        pwcsOldName,
        &mut currentEntry,
    );

    if currentEntryRef != DIRENTRY_NULL {
        if StorageBaseImpl_IsStreamOpen(This, currentEntryRef) != 0
            || StorageBaseImpl_IsStorageOpen(This, currentEntryRef) != 0
        {
            WARN!(b"Element is already open; cannot rename.\n\0".as_ptr() as PCHAR);
            return STG_E_ACCESSDENIED;
        }

        removeFromTree(This, (*This).storageDirEntry, currentEntryRef);

        lstrcpyW(currentEntry.name.as_mut_ptr(), pwcsNewName);

        currentEntry.leftChild = DIRENTRY_NULL;
        currentEntry.rightChild = DIRENTRY_NULL;

        StorageBaseImpl_WriteDirEntry(This, currentEntryRef, &mut currentEntry);

        insertIntoTree(This, (*This).storageDirEntry, currentEntryRef);
    } else {
        return STG_E_FILENOTFOUND;
    }

    StorageBaseImpl_Flush(This)
}
}

stdmethod! {
fn StorageBaseImpl_DestroyElement(
    iface: *mut IStorage,
    pwcsName: *const u16,
) -> HRESULT {
    let This = impl_from_IStorage(iface);

    let mut hr: HRESULT = 0;
    let mut entryToDelete: DirEntry = core::mem::zeroed();
    let entryToDeleteRef: DirRef;

    TRACE!(b"(%p, %s)\n\0".as_ptr() as PCHAR);

    if pwcsName.is_null() {
        return 0x80030009u32 as HRESULT;
    }

    if (*This).reverted != 0 {
        return 0x80030102u32 as HRESULT;
    }

    if ((*This).openFlags & 0x00010000) == 0 &&
       STGM_ACCESS_MODE((*This).openFlags) == 0x00000000 {
        return 0x80030005u32 as HRESULT;
    }

    entryToDeleteRef = findElement(
        This,
        (*This).storageDirEntry,
        pwcsName,
        &mut entryToDelete,
    );

    if entryToDeleteRef == 0xFFFFFFFF {
        TRACE!(b"<-- STG_E_FILENOTFOUND\n\0".as_ptr() as PCHAR);
        return 0x80030002u32 as HRESULT;
    }

    if entryToDelete.stgType == 1 {
        hr = deleteStorageContents(
            This,
            entryToDeleteRef,
            entryToDelete,
        );
    } else if entryToDelete.stgType == 2 {
        hr = deleteStreamContents(
            This,
            entryToDeleteRef,
            entryToDelete,
        );
    }

    if hr != 0 {
        TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
        return hr;
    }

    hr = removeFromTree(
        This,
        (*This).storageDirEntry,
        entryToDeleteRef,
    );

    if SUCCEEDED(hr) {
        StorageBaseImpl_DestroyDirEntry(This, entryToDeleteRef);
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_Flush(This);
    }

    TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
    hr
}
}

unsafe fn StorageImpl_GetNextBlockInChain(
    This: *mut StorageImpl,
    blockIndex: ULONG,
    nextBlockIndex: *mut ULONG,
) -> HRESULT {
    let offsetInDepot = blockIndex.wrapping_mul(core::mem::size_of::<ULONG>() as ULONG);
    let depotBlockCount = offsetInDepot / (*This).bigBlockSize;
    let depotBlockOffset = offsetInDepot % (*This).bigBlockSize;
    let mut depotBuffer: [BYTE; MAX_BIG_BLOCK_SIZE as usize] = core::mem::zeroed();
    let mut read: ULONG = 0;
    let depotBlockIndexPos: ULONG;
    let num_blocks: i32;

    *nextBlockIndex = BLOCK_SPECIAL;

    if depotBlockCount >= (*This).bigBlockDepotCount {
        
        WARN!(b"depotBlockCount %d, bigBlockDepotCount %d\n\0".as_ptr() as *const c_char);
        return STG_E_READFAULT;
    }

    if depotBlockCount != (*This).indexBlockDepotCached {
        (*This).indexBlockDepotCached = depotBlockCount;

        if depotBlockCount < COUNT_BBDEPOTINHEADER as ULONG {
            depotBlockIndexPos = (*This).bigBlockDepotStart[depotBlockCount as usize];
        } else {
            depotBlockIndexPos = Storage32Impl_GetExtDepotBlock(This, depotBlockCount);
        }

        StorageImpl_ReadBigBlock(
            This,
            depotBlockIndexPos,
            depotBuffer.as_mut_ptr() as *mut core::ffi::c_void,
            &mut read,
        );

        if read == 0 {
            return STG_E_READFAULT;
        }

        num_blocks = ((*This).bigBlockSize / 4) as i32;

        for index in 0..num_blocks {
            StorageUtl_ReadDWord(
                depotBuffer.as_ptr(),
                (index as ULONG).wrapping_mul(core::mem::size_of::<ULONG>() as ULONG),
                nextBlockIndex,
            );
            (*This).blockDepotCached[index as usize] = *nextBlockIndex;
        }
    }

    *nextBlockIndex = (*This).blockDepotCached
        [(depotBlockOffset / core::mem::size_of::<ULONG>() as ULONG) as usize];

    S_OK
}

unsafe fn deleteStorageContents(
    parentStorage: *mut StorageBaseImpl,
    indexToDelete: DirRef,
    entryDataToDelete: DirEntry,
) -> HRESULT {
    let mut elements: *mut IEnumSTATSTG = core::ptr::null_mut();
    let mut childStorage: *mut IStorage = core::ptr::null_mut();
    let mut currentElement: STATSTG = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut destroyHr: HRESULT = S_OK;
    
    TRACE!(b"%p,%d\n\0".as_ptr() as PCHAR);

    /* Invalidate any open storage objects. */
    // (rustconv emitted the macro as a CALL and then turned its body into a bogus
    //  `loop { ... if stg == NULL break }` — restored to the C's iteration.)
    LIST_FOR_EACH_ENTRY_SAFE!(
        stg,
        stg2,
        core::ptr::addr_of_mut!((*parentStorage).storageHead),
        StorageInternalImpl,
        ParentListEntry,
        {
            if (*stg).base.storageDirEntry == indexToDelete {
                StorageBaseImpl_Invalidate(core::ptr::addr_of_mut!((*stg).base));
            }
        }
    );

    hr = IStorage_OpenStorage(
        core::ptr::addr_of_mut!((*parentStorage).IStorage_iface),
        entryDataToDelete.name.as_ptr(),
        core::ptr::null_mut(),
        STGM_WRITE | STGM_SHARE_EXCLUSIVE,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(childStorage),
    );

    if hr != S_OK {
        TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
        return hr;
    }

    hr = IStorage_EnumElements(childStorage, 0, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(elements));
    if FAILED(hr) {
        IStorage_Release(childStorage);
        TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
        return hr;
    }

    loop {
        hr = IEnumSTATSTG_Next(elements, 1, core::ptr::addr_of_mut!(currentElement), core::ptr::null_mut());
        if hr == S_OK {
            destroyHr = IStorage_DestroyElement(childStorage, currentElement.pwcsName);
            CoTaskMemFree(currentElement.pwcsName as LPVOID);
        }

        IEnumSTATSTG_Reset(elements);

        if !(hr == S_OK && destroyHr == S_OK) {
            break;
        }
    }

    IStorage_Release(childStorage);
    IEnumSTATSTG_Release(elements);

    TRACE!(b"%08x\n\0".as_ptr() as PCHAR);
    destroyHr
}

unsafe fn StorageImpl_SetNextBlockInChain(
    This: *mut StorageImpl,
    blockIndex: ULONG,
    nextBlock: ULONG,
) {
    let offsetInDepot: ULONG = blockIndex.wrapping_mul(core::mem::size_of::<ULONG>() as ULONG);
    let depotBlockCount: ULONG = offsetInDepot / (*This).bigBlockSize;
    let depotBlockOffset: ULONG = offsetInDepot % (*This).bigBlockSize;
    let depotBlockIndexPos: ULONG;

    assert(depotBlockCount < (*This).bigBlockDepotCount);
    assert(blockIndex != nextBlock);

    if blockIndex == (RANGELOCK_FIRST / (*This).bigBlockSize).wrapping_sub(1) {
        ERR(b"Using range lock page\n\0".as_ptr() as PCHAR);
    }

    if depotBlockCount < COUNT_BBDEPOTINHEADER as ULONG {
        depotBlockIndexPos = (*This).bigBlockDepotStart[depotBlockCount as usize];
    } else {
        depotBlockIndexPos = Storage32Impl_GetExtDepotBlock(This, depotBlockCount);
    }

    StorageImpl_WriteDWordToBigBlock(
        This,
        depotBlockIndexPos,
        depotBlockOffset,
        nextBlock,
    );

    if depotBlockCount == (*This).indexBlockDepotCached {
        (*This).blockDepotCached[(depotBlockOffset / core::mem::size_of::<ULONG>() as ULONG) as usize] = nextBlock;
    }
}

unsafe fn BlockChainStream_WriteAt(
    This: *mut BlockChainStream,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *const core::ffi::c_void,
    bytesWritten: *mut ULONG,
) -> HRESULT {
    

    let mut blockNoInSequence: ULONG =
        (offset.QuadPart / (*(*This).parentStorage).bigBlockSize as ULONGLONG) as ULONG;
    let mut offsetInBlock: ULONG =
        (offset.QuadPart % (*(*This).parentStorage).bigBlockSize as ULONGLONG) as ULONG;
    let mut bytesToWrite: ULONG;
    let mut blockIndex: ULONG = 0;
    let bufferWalker_start: *const BYTE = buffer as *const BYTE;
    let mut bufferWalker: *const BYTE = bufferWalker_start;
    let mut hr: HRESULT;
    let mut cachedBlock: *mut BlockChainBlock = core::ptr::null_mut();
    let mut size_remaining: ULONG = size;

    *bytesWritten = 0;

    while size_remaining > 0 {
        let mut ulOffset: ULARGE_INTEGER = core::mem::zeroed();
        let mut bytesWrittenAt: DWORD = 0;

        bytesToWrite = min((*(*This).parentStorage).bigBlockSize - offsetInBlock, size_remaining);

        hr = BlockChainStream_GetBlockAtOffset(
            This,
            blockNoInSequence,
            &mut cachedBlock,
            &mut blockIndex,
            if size_remaining == bytesToWrite { 1 } else { 0 }
        );

        if FAILED(hr) {
            ERR(b"not enough blocks in chain to write data\n\0".as_ptr() as *const c_char);
            return hr;
        }

        if cachedBlock.is_null() {
            ulOffset.QuadPart = StorageImpl_GetBigBlockOffset((*This).parentStorage, blockIndex) + offsetInBlock as u64;

            StorageImpl_WriteAt(
                (*This).parentStorage,
                ulOffset,
                bufferWalker as *const core::ffi::c_void,
                bytesToWrite,
                &mut bytesWrittenAt
            );
        } else {
            if (*cachedBlock).read == 0 && bytesToWrite != (*(*This).parentStorage).bigBlockSize {
                let mut read: ULONG = 0;
                if FAILED(StorageImpl_ReadBigBlock(
                    (*This).parentStorage,
                    (*cachedBlock).sector,
                    (*cachedBlock).data.as_mut_ptr() as *mut core::ffi::c_void,
                    &mut read
                )) && read == 0 {
                    // C: STG_E_READFAULT (was 0x80030004, STG_E_TOOMANYOPENFILES)
                    return STG_E_READFAULT;
                }
            }

            core::ptr::copy_nonoverlapping(
                bufferWalker,
                (*cachedBlock).data.as_mut_ptr().add(offsetInBlock as usize),
                bytesToWrite as usize
            );
            bytesWrittenAt = bytesToWrite;
            (*cachedBlock).read = 1;
            (*cachedBlock).dirty = 1;
        }

        blockNoInSequence += 1;
        bufferWalker = bufferWalker.add(bytesWrittenAt as usize);
        size_remaining -= bytesWrittenAt;
        *bytesWritten += bytesWrittenAt;
        offsetInBlock = 0;

        if bytesWrittenAt != bytesToWrite {
            break;
        }
    }

    // C: STG_E_WRITEFAULT (was 0x80030004, STG_E_TOOMANYOPENFILES)
    if size_remaining == 0 { S_OK } else { STG_E_WRITEFAULT }
}

unsafe extern "C" fn StorageImpl_Flush(storage: *mut StorageBaseImpl) -> HRESULT {
    let This = storage as *mut StorageImpl;
    let mut i: i32;
    let mut hr: HRESULT;
    
    TRACE!(b"(%p)\n\0".as_ptr() as LPCSTR);
    
    hr = BlockChainStream_Flush((*This).smallBlockRootChain);
    
    if SUCCEEDED(hr) {
        hr = BlockChainStream_Flush((*This).rootBlockChain);
    }
    
    if SUCCEEDED(hr) {
        hr = BlockChainStream_Flush((*This).smallBlockDepotChain);
    }
    
    i = 0;
    while SUCCEEDED(hr) && i < BLOCKCHAIN_CACHE_SIZE as i32 {
        if !(*This).blockChainCache[i as usize].is_null() {
            hr = BlockChainStream_Flush((*This).blockChainCache[i as usize]);
        }
        i += 1;
    }
    
    if SUCCEEDED(hr) {
        hr = ILockBytes_Flush((*This).lockBytes);
    }
    
    hr
}

unsafe fn BlockChainStream_Destroy(This: *mut BlockChainStream) {
    if !This.is_null() {
        BlockChainStream_Flush(This);
        HeapFree(GetProcessHeap(), 0, (*This).indexCache as LPVOID);
    }
    HeapFree(GetProcessHeap(), 0, This as LPVOID);
}

unsafe extern "C" fn TransactedSnapshotImpl_WriteDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
    data: *const DirEntry,
) -> HRESULT {
    let This = base as *mut TransactedSnapshotImpl;
    let hr: HRESULT;

    

    TRACE!(b"%x %s l=%x r=%x d=%x\n\0".as_ptr() as *const i8);

    hr = TransactedSnapshotImpl_EnsureReadEntry(This, index);
    if FAILED(hr) {
        TRACE!(b"<-- %08x\n\0".as_ptr() as *const i8);
        return hr;
    }

    memcpy(
        core::ptr::addr_of_mut!((*(*This).entries.offset(index as isize)).data) as *mut core::ffi::c_void,
        data as *const core::ffi::c_void,
        core::mem::size_of::<DirEntry>(),
    );

    if index != (*This).base.storageDirEntry {
        (*(*This).entries.offset(index as isize)).dirty = 1;

        if (*data).size.QuadPart == 0
            && (*(*This).entries.offset(index as isize)).transactedParentEntry != DIRENTRY_NULL
        {
            let delete_ref: DirRef;
            delete_ref = TransactedSnapshotImpl_CreateStubEntry(
                This,
                (*(*This).entries.offset(index as isize)).transactedParentEntry,
            );

            if delete_ref != DIRENTRY_NULL {
                (*(*This).entries.offset(delete_ref as isize)).deleted = 1;
            }

            (*(*This).entries.offset(index as isize)).transactedParentEntry = DIRENTRY_NULL;
            (*(*This).entries.offset(index as isize)).newTransactedParentEntry = DIRENTRY_NULL;
        }
    }
    TRACE!(b"<-- S_OK\n\0".as_ptr() as *const i8);
    S_OK
}

unsafe extern "C" fn TransactedSnapshotImpl_ReadDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
    data: *mut DirEntry,
) -> HRESULT {
    let This = base as *mut TransactedSnapshotImpl;
    let hr = TransactedSnapshotImpl_EnsureReadEntry(This, index);
    if FAILED(hr) {
        TRACE!(b"<-- %08x\n\0".as_ptr() as LPCSTR);
        return hr;
    }

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*(*This).entries.offset(index as isize)).data),
        data,
        1,
    );

    TRACE!(b"%x %s l=%x r=%x d=%x\n\0".as_ptr() as LPCSTR);

    S_OK
}

unsafe extern "C" fn TransactedSnapshotImpl_StreamLink(
    base: *mut StorageBaseImpl,
    dst: DirRef,
    src: DirRef,
) -> HRESULT {
    let This = base as *mut TransactedSnapshotImpl;
    let mut hr: HRESULT;
    let dst_entry: *mut TransactedDirEntry;
    let src_entry: *mut TransactedDirEntry;

    hr = TransactedSnapshotImpl_EnsureReadEntry(This, src);
    if FAILED(hr) {
        TRACE!(b"<-- %08x\n\0".as_ptr() as LPCSTR);
        return hr;
    }

    hr = TransactedSnapshotImpl_EnsureReadEntry(This, dst);
    if FAILED(hr) {
        TRACE!(b"<-- %08x\n\0".as_ptr() as LPCSTR);
        return hr;
    }

    dst_entry = (*This).entries.offset(dst as isize);
    src_entry = (*This).entries.offset(src as isize);

    (*dst_entry).stream_dirty = (*src_entry).stream_dirty;
    (*dst_entry).stream_entry = (*src_entry).stream_entry;
    (*dst_entry).transactedParentEntry = (*src_entry).transactedParentEntry;
    (*dst_entry).newTransactedParentEntry = (*src_entry).newTransactedParentEntry;
    (*dst_entry).data.size = (*src_entry).data.size;

    S_OK
}

unsafe extern "C" fn TransactedSnapshotImpl_StreamWriteAt(
    base: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *const core::ffi::c_void,
    bytesWritten: *mut ULONG,
) -> HRESULT {
    let This = base as *mut TransactedSnapshotImpl;
    let mut hr: HRESULT;

    hr = TransactedSnapshotImpl_EnsureReadEntry(This, index);
    if FAILED(hr) {
        TRACE!(b"<-- %08x\n\0".as_ptr() as LPCSTR);
        return hr;
    }

    hr = TransactedSnapshotImpl_MakeStreamDirty(This, index);
    if FAILED(hr) {
        TRACE!(b"<-- %08x\n\0".as_ptr() as LPCSTR);
        return hr;
    }

    hr = StorageBaseImpl_StreamWriteAt(
        (*This).scratch,
        (*(*This).entries.offset(index as isize)).stream_entry,
        offset,
        size,
        buffer,
        bytesWritten,
    );

    if SUCCEEDED(hr) && size != 0 {
        let current_size = (*(*This).entries.offset(index as isize)).data.size.QuadPart;
        let new_size = offset.QuadPart + size as u64;
        (*(*This).entries.offset(index as isize)).data.size.QuadPart = max(current_size, new_size);
    }

    TRACE!(b"<-- %08x\n\0".as_ptr() as LPCSTR);
    hr
}

unsafe extern "C" fn TransactedSnapshotImpl_StreamSetSize(
    base: *mut StorageBaseImpl,
    index: DirRef,
    newsize: ULARGE_INTEGER,
) -> HRESULT {
    let This = base as *mut TransactedSnapshotImpl;
    let mut hr: HRESULT;

    hr = TransactedSnapshotImpl_EnsureReadEntry(This, index);
    if FAILED(hr) {
        TRACE!(b"<-- %08x\n\0".as_ptr() as LPCSTR);
        return hr;
    }

    if (*(*This).entries.offset(index as isize)).data.size.QuadPart == newsize.QuadPart {
        return 0x00000000;
    }

    if newsize.QuadPart == 0 {
        if (*(*This).entries.offset(index as isize)).stream_dirty != 0 {
            let zero = ULARGE_INTEGER { QuadPart: 0 };
            StorageBaseImpl_StreamSetSize(
                (*This).scratch,
                (*(*This).entries.offset(index as isize)).stream_entry,
                zero,
            );
            StorageBaseImpl_DestroyDirEntry(
                (*This).scratch,
                (*(*This).entries.offset(index as isize)).stream_entry,
            );
            (*(*This).entries.offset(index as isize)).stream_dirty = 0;
        } else if (*(*This).entries.offset(index as isize)).transactedParentEntry != 0xFFFFFFFF {
            let delete_ref = TransactedSnapshotImpl_CreateStubEntry(
                This,
                (*(*This).entries.offset(index as isize)).transactedParentEntry,
            );

            if delete_ref != 0xFFFFFFFF {
                (*(*This).entries.offset(delete_ref as isize)).deleted = 1;
            }

            (*(*This).entries.offset(index as isize)).transactedParentEntry = 0xFFFFFFFF;
            (*(*This).entries.offset(index as isize)).newTransactedParentEntry = 0xFFFFFFFF;
        }
    } else {
        hr = TransactedSnapshotImpl_MakeStreamDirty(This, index);
        if FAILED(hr) {
            return hr;
        }

        hr = StorageBaseImpl_StreamSetSize(
            (*This).scratch,
            (*(*This).entries.offset(index as isize)).stream_entry,
            newsize,
        );
    }

    if SUCCEEDED(hr) {
        (*(*This).entries.offset(index as isize)).data.size = newsize;
    }

    TRACE!(b"<-- %08x\n\0".as_ptr() as LPCSTR);
    hr
}

unsafe fn TransactedSnapshotImpl_CopyTree(This: *mut TransactedSnapshotImpl) -> HRESULT {
    let mut cursor: DirRef;
    let mut entry: *mut TransactedDirEntry;
    let mut hr: HRESULT = S_OK;

    cursor = (*This).base.storageDirEntry;
    entry = (*This).entries.offset((cursor as usize) as isize);
    (*entry).parent = DIRENTRY_NULL;
    (*entry).newTransactedParentEntry = (*entry).transactedParentEntry;

    if (*entry).data.dirRootEntry == DIRENTRY_NULL {
        return S_OK;
    }

    (*(*This).entries.offset(((*entry).data.dirRootEntry as usize) as isize)).parent = DIRENTRY_NULL;

    cursor = TransactedSnapshotImpl_FindFirstChild(This, (*entry).data.dirRootEntry);
    entry = (*This).entries.offset((cursor as usize) as isize);

    while cursor != DIRENTRY_NULL {
        if (*entry).read == 0
            || ((*entry).dirty == 0
                && (*entry).stream_dirty == 0
                && TransactedSnapshotImpl_MadeCopy(This, (*entry).data.leftChild) == 0
                && TransactedSnapshotImpl_MadeCopy(This, (*entry).data.rightChild) == 0
                && TransactedSnapshotImpl_MadeCopy(This, (*entry).data.dirRootEntry) == 0)
        {
            (*entry).newTransactedParentEntry = (*entry).transactedParentEntry;
        } else {
            let mut newData: DirEntry = core::mem::zeroed();

            core::ptr::copy_nonoverlapping(
                &(*entry).data as *const DirEntry,
                &mut newData as *mut DirEntry,
                1,
            );

            newData.size.QuadPart = 0;
            newData.startingBlock = BLOCK_END_OF_CHAIN;

            if newData.leftChild != DIRENTRY_NULL {
                newData.leftChild =
                    (*(*This).entries.offset((newData.leftChild as usize) as isize)).newTransactedParentEntry;
            }

            if newData.rightChild != DIRENTRY_NULL {
                newData.rightChild =
                    (*(*This).entries.offset((newData.rightChild as usize) as isize)).newTransactedParentEntry;
            }

            if newData.dirRootEntry != DIRENTRY_NULL {
                newData.dirRootEntry =
                    (*(*This).entries.offset((newData.dirRootEntry as usize) as isize)).newTransactedParentEntry;
            }

            hr = StorageBaseImpl_CreateDirEntry(
                (*This).transactedParent,
                &mut newData,
                &mut (*entry).newTransactedParentEntry,
            );
            if FAILED(hr) {
                TransactedSnapshotImpl_DestroyTemporaryCopy(This, cursor);
                return hr;
            }

            if (*entry).stream_dirty != 0 {
                hr = StorageBaseImpl_CopyStream(
                    (*This).transactedParent,
                    (*entry).newTransactedParentEntry,
                    (*This).scratch,
                    (*entry).stream_entry,
                );
            } else if (*entry).data.size.QuadPart != 0 {
                hr = StorageBaseImpl_StreamLink(
                    (*This).transactedParent,
                    (*entry).newTransactedParentEntry,
                    (*entry).transactedParentEntry,
                );
            }

            if FAILED(hr) {
                cursor = TransactedSnapshotImpl_FindNextChild(This, cursor);
                TransactedSnapshotImpl_DestroyTemporaryCopy(This, cursor);
                return hr;
            }
        }

        cursor = TransactedSnapshotImpl_FindNextChild(This, cursor);
        entry = (*This).entries.offset((cursor as usize) as isize);
    }

    hr
}

unsafe fn deleteStreamContents(
    parentStorage: *mut StorageBaseImpl,
    indexToDelete: DirRef,
    entryDataToDelete: DirEntry,
) -> HRESULT {
    let mut pis: *mut IStream = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut size: ULARGE_INTEGER = core::mem::zeroed();

    /* Invalidate any open stream objects. */
    LIST_FOR_EACH_ENTRY_SAFE!(
        strm,
        strm2,
        core::ptr::addr_of_mut!((*parentStorage).strmHead),
        StgStreamImpl,
        StrmListEntry,
        {
            if (*strm).dirEntry == indexToDelete {
                TRACE!(b"Stream deleted %p\n\0".as_ptr() as PCHAR);
                (*strm).parentStorage = core::ptr::null_mut();
                list_remove(core::ptr::addr_of_mut!((*strm).StrmListEntry));
            }
        }
    );

    size.u.HighPart = 0;
    size.u.LowPart = 0;

    hr = IStorage_OpenStream(
        core::ptr::addr_of_mut!((*parentStorage).IStorage_iface),
        entryDataToDelete.name.as_ptr(),
        core::ptr::null_mut(),
        STGM_WRITE | STGM_SHARE_EXCLUSIVE,
        0,
        &mut pis,
    );

    if hr != S_OK {
        TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
        return hr;
    }

    hr = IStream_SetSize(pis, size);

    if hr != S_OK {
        TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
        return hr;
    }

    IStream_Release(pis);
    TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
    S_OK
}

unsafe fn StorageImpl_GetNextFreeBigBlock(This: *mut StorageImpl, neededAddNumBlocks: ULONG) -> ULONG {
    let mut depotBlockIndexPos: ULONG;
    let mut depotBuffer: [BYTE; MAX_BIG_BLOCK_SIZE as usize] = [0; MAX_BIG_BLOCK_SIZE as usize];
    let mut depotBlockOffset: ULONG;
    let blocksPerDepot: ULONG = (*This).bigBlockSize / core::mem::size_of::<ULONG>() as ULONG;
    let mut nextBlockIndex: ULONG = BLOCK_SPECIAL;
    let mut depotIndex: i32;
    let mut freeBlock: ULONG = BLOCK_UNUSED;
    let mut read: ULONG = 0;
    let mut neededSize: ULARGE_INTEGER = core::mem::zeroed();
    let mut statstg: STATSTG = core::mem::zeroed();

    depotIndex = ((*This).prevFreeBlock / blocksPerDepot) as i32;
    depotBlockOffset = ((*This).prevFreeBlock % blocksPerDepot) * core::mem::size_of::<ULONG>() as ULONG;

    while nextBlockIndex != BLOCK_UNUSED {
        if depotIndex < COUNT_BBDEPOTINHEADER as i32 {
            depotBlockIndexPos = (*This).bigBlockDepotStart[depotIndex as usize];

            if depotBlockIndexPos == BLOCK_UNUSED {
                depotBlockIndexPos = depotIndex as ULONG * blocksPerDepot;

                Storage32Impl_AddBlockDepot(This, depotBlockIndexPos, depotIndex as ULONG);
                (*This).bigBlockDepotCount += 1;
                (*This).bigBlockDepotStart[depotIndex as usize] = depotBlockIndexPos;

                StorageImpl_SetNextBlockInChain(This, depotBlockIndexPos, BLOCK_SPECIAL);

                StorageImpl_SaveFileHeader(This);
            }
        } else {
            depotBlockIndexPos = Storage32Impl_GetExtDepotBlock(This, depotIndex as ULONG);

            if depotBlockIndexPos == BLOCK_UNUSED {
                let mut extIndex: ULONG = BLOCK_UNUSED;
                let numExtBlocks: ULONG = depotIndex as ULONG - COUNT_BBDEPOTINHEADER as ULONG;
                let extBlockOffset: ULONG = numExtBlocks % (blocksPerDepot - 1);

                if extBlockOffset == 0 {
                    extIndex = Storage32Impl_AddExtBlockDepot(This);
                    (*This).extBigBlockDepotCount += 1;
                    depotBlockIndexPos = extIndex + 1;
                } else {
                    depotBlockIndexPos = depotIndex as ULONG * blocksPerDepot;
                }

                Storage32Impl_AddBlockDepot(This, depotBlockIndexPos, depotIndex as ULONG);
                (*This).bigBlockDepotCount += 1;
                Storage32Impl_SetExtDepotBlock(This, depotIndex as ULONG, depotBlockIndexPos);

                StorageImpl_SetNextBlockInChain(This, depotBlockIndexPos, BLOCK_SPECIAL);

                if extIndex != BLOCK_UNUSED {
                    StorageImpl_SetNextBlockInChain(This, extIndex, BLOCK_EXTBBDEPOT);
                }

                StorageImpl_SaveFileHeader(This);
            }
        }

        StorageImpl_ReadBigBlock(This, depotBlockIndexPos, depotBuffer.as_mut_ptr() as *mut core::ffi::c_void, &mut read);

        if read != 0 {
            while (depotBlockOffset / core::mem::size_of::<ULONG>() as ULONG) < blocksPerDepot
                && nextBlockIndex != BLOCK_UNUSED
            {
                StorageUtl_ReadDWord(depotBuffer.as_ptr(), depotBlockOffset, &mut nextBlockIndex);

                if nextBlockIndex == BLOCK_UNUSED {
                    freeBlock = (depotIndex as ULONG * blocksPerDepot)
                        + (depotBlockOffset / core::mem::size_of::<ULONG>() as ULONG);
                }

                depotBlockOffset += core::mem::size_of::<ULONG>() as ULONG;
            }
        }

        depotIndex += 1;
        depotBlockOffset = 0;
    }

    // wine-10: grow the file once for every block the caller is about to add, not
    // once per block. u64 product: Wine's ULONG * ULONG could wrap.
    neededSize.QuadPart = StorageImpl_GetBigBlockOffset(This, freeBlock)
        + (*This).bigBlockSize as u64 * neededAddNumBlocks as u64;

    ILockBytes_Stat((*This).lockBytes, &mut statstg, STATFLAG_NONAME);

    if neededSize.QuadPart > statstg.cbSize.QuadPart {
        ILockBytes_SetSize((*This).lockBytes, neededSize);
    }

    (*This).prevFreeBlock = freeBlock;

    freeBlock
}

unsafe fn StorageImpl_FreeBigBlock(
    This: *mut StorageImpl,
    blockIndex: ULONG,
) {
    StorageImpl_SetNextBlockInChain(This, blockIndex, BLOCK_UNUSED);

    if blockIndex < (*This).prevFreeBlock {
        (*This).prevFreeBlock = blockIndex;
    }
}

unsafe fn StorageImpl_WriteRawDirEntry(
    This: *mut StorageImpl,
    index: ULONG,
    buffer: *const BYTE,
) -> HRESULT {
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut bytesRead: ULONG = 0;

    offset.QuadPart = (index as ULONGLONG) * RAW_DIRENTRY_SIZE as ULONGLONG;

    return BlockChainStream_WriteAt(
        (*This).rootBlockChain,
        offset,
        RAW_DIRENTRY_SIZE,
        buffer as *const core::ffi::c_void,
        &mut bytesRead,
    );
}

unsafe fn SmallBlockChainStream_SetNextBlockInChain(
    This: *mut SmallBlockChainStream,
    blockIndex: ULONG,
    nextBlock: ULONG,
) {
    let mut offsetOfBlockInDepot: ULARGE_INTEGER = core::mem::zeroed();
    let mut buffer: DWORD = 0;
    let mut bytesWritten: ULONG = 0;

    offsetOfBlockInDepot.QuadPart = (blockIndex as ULONGLONG)
        .wrapping_mul(core::mem::size_of::<ULONG>() as ULONGLONG);

    StorageUtl_WriteDWord(
        &mut buffer as *mut DWORD as *mut BYTE,
        0,
        nextBlock,
    );

    BlockChainStream_WriteAt(
        (*(*This).parentStorage).smallBlockDepotChain,
        offsetOfBlockInDepot,
        core::mem::size_of::<DWORD>() as ULONG,
        &buffer as *const DWORD as *const core::ffi::c_void,
        &mut bytesWritten,
    );
}

/// storage32.c:4339 — rustconv dropped this one (it returns `BlockChainStream **`);
/// translated by hand from the C.
unsafe fn StorageImpl_GetFreeBlockChainCacheEntry(
    This: *mut StorageImpl,
) -> *mut *mut BlockChainStream {
    let mut i: usize = 0;

    while i < BLOCKCHAIN_CACHE_SIZE {
        if (*This).blockChainCache[i].is_null() {
            return core::ptr::addr_of_mut!((*This).blockChainCache[i]);
        }
        i += 1;
    }

    let i = (*This).blockChainToEvict as usize;

    BlockChainStream_Destroy((*This).blockChainCache[i]);
    (*This).blockChainCache[i] = core::ptr::null_mut();

    (*This).blockChainToEvict += 1;
    if (*This).blockChainToEvict == BLOCKCHAIN_CACHE_SIZE as UINT {
        (*This).blockChainToEvict = 0;
    }

    core::ptr::addr_of_mut!((*This).blockChainCache[i])
}

/// storage32.c:4363 — likewise dropped by rustconv; translated by hand from the C.
unsafe fn StorageImpl_GetCachedBlockChainStream(
    This: *mut StorageImpl,
    index: DirRef,
) -> *mut *mut BlockChainStream {
    let mut i: i32 = 0;
    let mut free_index: i32 = -1;

    while i < BLOCKCHAIN_CACHE_SIZE as i32 {
        if (*This).blockChainCache[i as usize].is_null() {
            if free_index == -1 {
                free_index = i;
            }
        } else if (*(*This).blockChainCache[i as usize]).ownerDirEntry == index {
            return core::ptr::addr_of_mut!((*This).blockChainCache[i as usize]);
        }
        i += 1;
    }

    if free_index == -1 {
        free_index = (*This).blockChainToEvict as i32;

        BlockChainStream_Destroy((*This).blockChainCache[free_index as usize]);
        (*This).blockChainCache[free_index as usize] = core::ptr::null_mut();

        (*This).blockChainToEvict += 1;
        if (*This).blockChainToEvict == BLOCKCHAIN_CACHE_SIZE as UINT {
            (*This).blockChainToEvict = 0;
        }
    }

    (*This).blockChainCache[free_index as usize] =
        BlockChainStream_Construct(This, core::ptr::null_mut(), index);
    core::ptr::addr_of_mut!((*This).blockChainCache[free_index as usize])
}

unsafe fn StorageImpl_DeleteCachedBlockChainStream(This: *mut StorageImpl, index: DirRef) {
    let mut i: usize = 0;

    while i < BLOCKCHAIN_CACHE_SIZE {
        if !(*This).blockChainCache[i as usize].is_null() 
            && (*(*This).blockChainCache[i as usize]).ownerDirEntry == index
        {
            BlockChainStream_Destroy((*This).blockChainCache[i as usize]);
            (*This).blockChainCache[i as usize] = core::ptr::null_mut();
            return;
        }
        i += 1;
    }
}

unsafe extern "C" fn StorageImpl_Destroy(iface: *mut StorageBaseImpl) {
    let This = iface as *mut StorageImpl;
    let mut i: i32;
    
    TRACE!(b"(%p)\n\0".as_ptr() as PCHAR);
    
    StorageImpl_Flush(iface);
    
    StorageImpl_Invalidate(iface);
    
    HeapFree(GetProcessHeap(), 0, (*This).extBigBlockDepotLocations as LPVOID);
    
    BlockChainStream_Destroy((*This).smallBlockRootChain);
    BlockChainStream_Destroy((*This).rootBlockChain);
    BlockChainStream_Destroy((*This).smallBlockDepotChain);
    
    i = 0;
    while i < BLOCKCHAIN_CACHE_SIZE as i32 {
        BlockChainStream_Destroy((*This).blockChainCache[i as usize]);
        i += 1;
    }
    
    i = 0;
    while i < (*This).locked_bytes.len() as i32 {
        let mut offset: ULARGE_INTEGER = core::mem::zeroed();
        let mut cb: ULARGE_INTEGER = core::mem::zeroed();
        cb.QuadPart = 1;
        if (*This).locked_bytes[i as usize] != 0 {
            offset.QuadPart = (*This).locked_bytes[i as usize] as ULONGLONG;
            StorageImpl_UnlockRegion(This, offset, cb, LOCK_ONLYONCE);
        }
        i += 1;
    }
    
    if !(*This).lockBytes.is_null() {
        ILockBytes_Release((*This).lockBytes);
    }
    HeapFree(GetProcessHeap(), 0, This as LPVOID);
}

stdmethod! {
fn TransactedSnapshotImpl_Commit(
    iface: *mut IStorage,
    grfCommitFlags: DWORD,
) -> HRESULT {
    let This = impl_from_IStorage(iface) as *mut TransactedSnapshotImpl;
    let root_entry: *mut TransactedDirEntry;
    let mut i: DirRef;
    let dir_root_ref: DirRef;
    let mut data: DirEntry = core::mem::zeroed();
    let mut zero: ULARGE_INTEGER = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut transactionSig: ULONG = 0;

    zero.QuadPart = 0;

    TRACE!(b"(%p,%x)\n\0".as_ptr() as PCHAR);

    if STGM_ACCESS_MODE((*This).base.openFlags) == STGM_READ {
        return STG_E_ACCESSDENIED;
    }

    hr = StorageBaseImpl_LockTransaction((*This).transactedParent, 1);
    if hr == E_NOTIMPL {
        hr = S_OK;
    }
    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_GetTransactionSig(
            (*This).transactedParent,
            core::ptr::addr_of_mut!(transactionSig),
            1,
        );
        if SUCCEEDED(hr) {
            if transactionSig != (*This).lastTransactionSig {
                ERR(b"file was externally modified\n\0".as_ptr() as PCHAR);
                hr = STG_E_NOTCURRENT;
            }

            if SUCCEEDED(hr) {
                (*This).lastTransactionSig = transactionSig.wrapping_add(1);
                hr = StorageBaseImpl_SetTransactionSig(
                    (*This).transactedParent,
                    (*This).lastTransactionSig,
                );
            }
        } else if hr == E_NOTIMPL {
            hr = S_OK;
        }

        if FAILED(hr) {
            StorageBaseImpl_UnlockTransaction((*This).transactedParent, 1);
            TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
            return hr;
        }

        root_entry = ((*This).entries).offset((*This).base.storageDirEntry as isize);

        if (*root_entry).read == 0 {
            StorageBaseImpl_UnlockTransaction((*This).transactedParent, 1);
            TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
            return hr;
        }

        hr = TransactedSnapshotImpl_CopyTree(This);
        if FAILED(hr) {
            StorageBaseImpl_UnlockTransaction((*This).transactedParent, 1);
            TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
            return hr;
        }

        if (*root_entry).data.dirRootEntry == DIRENTRY_NULL {
            dir_root_ref = DIRENTRY_NULL;
        } else {
            dir_root_ref = (*((*This).entries).offset((*root_entry).data.dirRootEntry as isize))
                .newTransactedParentEntry;
        }

        hr = StorageBaseImpl_Flush((*This).transactedParent);

        if SUCCEEDED(hr) {
            hr = StorageBaseImpl_ReadDirEntry(
                (*This).transactedParent,
                (*root_entry).transactedParentEntry,
                core::ptr::addr_of_mut!(data),
            );
        }

        if SUCCEEDED(hr) {
            data.dirRootEntry = dir_root_ref;
            data.clsid = (*root_entry).data.clsid;
            data.ctime = (*root_entry).data.ctime;
            data.mtime = (*root_entry).data.mtime;

            hr = StorageBaseImpl_WriteDirEntry(
                (*This).transactedParent,
                (*root_entry).transactedParentEntry,
                core::ptr::addr_of_mut!(data),
            );
        }

        StorageBaseImpl_Flush((*This).transactedParent);

        if SUCCEEDED(hr) {
            i = 0;
            while i < (*This).entries_size {
                let entry = ((*This).entries).offset(i as isize);
                if (*entry).inuse != 0 {
                    if (*entry).deleted != 0 {
                        StorageBaseImpl_StreamSetSize(
                            (*This).transactedParent,
                            (*entry).transactedParentEntry,
                            zero,
                        );
                        StorageBaseImpl_DestroyDirEntry(
                            (*This).transactedParent,
                            (*entry).transactedParentEntry,
                        );
                        memset(
                            entry as *mut core::ffi::c_void,
                            0,
                            core::mem::size_of::<TransactedDirEntry>(),
                        );
                        (*This).firstFreeEntry = min(i, (*This).firstFreeEntry);
                    } else if (*entry).read != 0
                        && (*entry).transactedParentEntry != (*entry).newTransactedParentEntry
                    {
                        if (*entry).transactedParentEntry != DIRENTRY_NULL {
                            StorageBaseImpl_DestroyDirEntry(
                                (*This).transactedParent,
                                (*entry).transactedParentEntry,
                            );
                        }
                        if (*entry).stream_dirty != 0 {
                            StorageBaseImpl_StreamSetSize(
                                (*This).scratch,
                                (*entry).stream_entry,
                                zero,
                            );
                            StorageBaseImpl_DestroyDirEntry(
                                (*This).scratch,
                                (*entry).stream_entry,
                            );
                            (*entry).stream_dirty = 0;
                        }
                        (*entry).dirty = 0;
                        (*entry).transactedParentEntry = (*entry).newTransactedParentEntry;
                    }
                }
                i = i.wrapping_add(1);
            }
        } else {
            TransactedSnapshotImpl_DestroyTemporaryCopy(This, DIRENTRY_NULL);
        }

        if SUCCEEDED(hr) {
            hr = StorageBaseImpl_Flush((*This).transactedParent);
        }
        StorageBaseImpl_UnlockTransaction((*This).transactedParent, 1);
    }

    TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
    return hr;
}
}

unsafe extern "C" fn StorageImpl_DestroyDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
) -> HRESULT {
    let mut emptyData: [BYTE; RAW_DIRENTRY_SIZE as usize] = [0; RAW_DIRENTRY_SIZE as usize];
    let storage = base as *mut StorageImpl;

    core::ptr::write_bytes(emptyData.as_mut_ptr(), 0, RAW_DIRENTRY_SIZE as usize);

    StorageImpl_WriteRawDirEntry(storage, index, emptyData.as_ptr())
}

unsafe fn StorageImpl_WriteDirEntry(
    This: *mut StorageImpl,
    index: DirRef,
    buffer: *const DirEntry,
) -> HRESULT {
    let mut currentEntry: [BYTE; RAW_DIRENTRY_SIZE as usize] = [0; RAW_DIRENTRY_SIZE as usize];

    UpdateRawDirEntry(currentEntry.as_mut_ptr(), buffer);

    StorageImpl_WriteRawDirEntry(This, index, currentEntry.as_ptr())
}

unsafe fn SmallBlockChainStream_FreeBlock(
    This: *mut SmallBlockChainStream,
    blockIndex: ULONG,
) {
    SmallBlockChainStream_SetNextBlockInChain(This, blockIndex, BLOCK_UNUSED);
}

unsafe fn StorageBaseImpl_DeleteAll(stg: *mut StorageBaseImpl) {
    LIST_FOR_EACH_ENTRY_SAFE!(
        strm,
        cur2,
        addr_of_mut!((*stg).strmHead),
        StgStreamImpl,
        StrmListEntry,
        {
            TRACE!(b"Streams invalidated (stg=%p strm=%p next=%p prev=%p)\n\0".as_ptr() as LPCSTR);
            (*strm).parentStorage = core::ptr::null_mut();
            list_remove(addr_of_mut!((*strm).StrmListEntry));
        }
    );

    LIST_FOR_EACH_ENTRY_SAFE!(
        childstg,
        cur2,
        addr_of_mut!((*stg).storageHead),
        StorageInternalImpl,
        ParentListEntry,
        {
            StorageBaseImpl_Invalidate(addr_of_mut!((*childstg).base));
        }
    );

    if !(*stg).transactedChild.is_null() {
        StorageBaseImpl_Invalidate((*stg).transactedChild);
        (*stg).transactedChild = core::ptr::null_mut();
    }
}

unsafe extern "C" fn StorageImpl_BaseWriteDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
    data: *const DirEntry,
) -> HRESULT {
    let This = base as *mut StorageImpl;
    StorageImpl_WriteDirEntry(This, index, data)
}

unsafe fn StorageBaseImpl_CopyChildEntryTo(
    This: *mut StorageBaseImpl,
    srcEntry: DirRef,
    skip_storage: BOOL,
    skip_stream: BOOL,
    snbExclude: SNB,
    pstgDest: *mut IStorage,
) -> HRESULT {
    let mut data: DirEntry = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut skip: BOOL = 0;
    let mut pstgTmp: *mut IStorage = core::ptr::null_mut();
    let mut pstrChild: *mut IStream = core::ptr::null_mut();
    let mut pstrTmp: *mut IStream = core::ptr::null_mut();
    let mut strStat: STATSTG = core::mem::zeroed();

    if srcEntry == DIRENTRY_NULL {
        return S_OK;
    }

    hr = StorageBaseImpl_ReadDirEntry(This, srcEntry, &mut data);

    if FAILED(hr) {
        return hr;
    }

    if !snbExclude.is_null() {
        let mut snb = snbExclude;

        while !(*snb).is_null() && skip == 0 {
            if wcscmp(data.name.as_ptr(), *snb) == 0 {
                skip = 1;
            }
            snb = snb.offset(1);
        }
    }

    if skip == 0 {
        if data.stgType as DWORD == STGTY_STORAGE && skip_storage == 0 {
            hr = IStorage_CreateStorage(
                pstgDest,
                data.name.as_ptr(),
                STGM_FAILIFTHERE | STGM_WRITE | STGM_SHARE_EXCLUSIVE,
                0,
                0,
                &mut pstgTmp,
            );

            if hr == STG_E_FILEALREADYEXISTS {
                hr = IStorage_OpenStorage(
                    pstgDest,
                    data.name.as_ptr(),
                    core::ptr::null_mut(),
                    STGM_WRITE | STGM_SHARE_EXCLUSIVE,
                    core::ptr::null_mut(),
                    0,
                    &mut pstgTmp,
                );
            }

            if SUCCEEDED(hr) {
                hr = StorageBaseImpl_CopyStorageEntryTo(
                    This,
                    srcEntry,
                    skip_storage,
                    skip_stream,
                    core::ptr::null_mut(),
                    pstgTmp,
                );

                IStorage_Release(pstgTmp);
            }
        } else if data.stgType as DWORD == STGTY_STREAM && skip_stream == 0 {
            hr = IStorage_CreateStream(
                pstgDest,
                data.name.as_ptr(),
                STGM_CREATE | STGM_WRITE | STGM_SHARE_EXCLUSIVE,
                0,
                0,
                &mut pstrTmp,
            );

            if hr == S_OK {
                let streamimpl = StgStreamImpl_Construct(
                    This,
                    STGM_READ | STGM_SHARE_EXCLUSIVE,
                    srcEntry,
                );

                if !streamimpl.is_null() {
                    pstrChild = &mut (*streamimpl).IStream_iface;
                    if !pstrChild.is_null() {
                        IStream_AddRef(pstrChild);
                    }
                } else {
                    pstrChild = core::ptr::null_mut();
                    hr = E_OUTOFMEMORY;
                }
            }

            if hr == S_OK {
                IStream_Stat(pstrChild, &mut strStat, STATFLAG_NONAME);

                IStream_SetSize(pstrTmp, strStat.cbSize);

                hr = IStream_CopyTo(
                    pstrChild,
                    pstrTmp,
                    strStat.cbSize,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );

                IStream_Release(pstrChild);
            }

            IStream_Release(pstrTmp);
        }
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_CopyChildEntryTo(
            This,
            data.leftChild,
            skip_storage,
            skip_stream,
            snbExclude,
            pstgDest,
        );
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_CopyChildEntryTo(
            This,
            data.rightChild,
            skip_storage,
            skip_stream,
            snbExclude,
            pstgDest,
        );
    }

    TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
    hr
}

stdmethod! {
fn StorageBaseImpl_OpenStorage(
    iface: *mut IStorage,
    pwcsName: *const OLECHAR,
    pstgPriority: *mut IStorage,
    grfMode: DWORD,
    snbExclude: SNB,
    reserved: DWORD,
    ppstg: *mut *mut IStorage,
) -> HRESULT {
    let This = impl_from_IStorage(iface);
    let newStorage: *mut StorageInternalImpl;
    let mut newTransactedStorage: *mut StorageBaseImpl = core::ptr::null_mut();
    let mut currentEntry: DirEntry = core::mem::zeroed();
    let storageEntryRef: DirRef;
    let mut res: HRESULT;

    TRACE!(b"(%p, %s, %p, %x, %p, %d, %p)\n\0".as_ptr() as PCHAR);

    'end: loop {
        if pwcsName.is_null() || ppstg.is_null() {
            res = E_INVALIDARG;
            break 'end;
        }

        if (*This).openFlags & STGM_SIMPLE != 0 {
            res = STG_E_INVALIDFUNCTION;
            break 'end;
        }

        if !snbExclude.is_null() {
            res = STG_E_INVALIDPARAMETER;
            break 'end;
        }

        if FAILED(validateSTGM(grfMode)) {
            res = STG_E_INVALIDFLAG;
            break 'end;
        }

        if STGM_SHARE_MODE(grfMode) != STGM_SHARE_EXCLUSIVE
            || (grfMode & STGM_DELETEONRELEASE) != 0
            || (grfMode & STGM_PRIORITY) != 0
        {
            res = STG_E_INVALIDFUNCTION;
            break 'end;
        }

        if (*This).reverted != 0 {
            return STG_E_REVERTED;
        }

        if ((*This).openFlags & STGM_TRANSACTED) == 0 {
            if STGM_ACCESS_MODE(grfMode) > STGM_ACCESS_MODE((*This).openFlags) {
                res = STG_E_ACCESSDENIED;
                break 'end;
            }
        }

        *ppstg = core::ptr::null_mut();

        storageEntryRef = findElement(
            This,
            (*This).storageDirEntry,
            pwcsName,
            &mut currentEntry,
        );

        if storageEntryRef != DIRENTRY_NULL && currentEntry.stgType as DWORD == STGTY_STORAGE {
            if StorageBaseImpl_IsStorageOpen(This, storageEntryRef) != 0 {
                res = STG_E_ACCESSDENIED;
                break 'end;
            }

            newStorage = StorageInternalImpl_Construct(This, grfMode, storageEntryRef);

            if !newStorage.is_null() {
                if (grfMode & STGM_TRANSACTED) != 0 {
                    res = Storage_ConstructTransacted(
                        &mut (*newStorage).base,
                        0,
                        &mut newTransactedStorage,
                    );

                    if FAILED(res) {
                        HeapFree(GetProcessHeap(), 0, newStorage as LPVOID);
                        break 'end;
                    }

                    *ppstg = &mut (*newTransactedStorage).IStorage_iface;
                } else {
                    *ppstg = &mut (*newStorage).base.IStorage_iface;
                }

                list_add_tail(
                    &mut (*This).storageHead,
                    &mut (*newStorage).ParentListEntry,
                );

                res = S_OK;
                break 'end;
            }

            res = STG_E_INSUFFICIENTMEMORY;
            break 'end;
        }

        res = STG_E_FILENOTFOUND;
        break 'end;
    }

    TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
    return res;
}
}

unsafe fn StorageBaseImpl_CopyStorageEntryTo(
    This: *mut StorageBaseImpl,
    srcEntry: DirRef,
    skip_storage: BOOL,
    skip_stream: BOOL,
    snbExclude: SNB,
    pstgDest: *mut IStorage,
) -> HRESULT {
    let mut data: DirEntry = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = StorageBaseImpl_ReadDirEntry(This, srcEntry, &mut data);

    if SUCCEEDED(hr) {
        hr = IStorage_SetClass(pstgDest, &data.clsid);
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_CopyChildEntryTo(
            This,
            data.dirRootEntry,
            skip_storage,
            skip_stream,
            snbExclude,
            pstgDest,
        );
    }

    TRACE!(b"<-- %08x\n\0".as_ptr() as PCHAR);
    hr
}

stdmethod! {
fn StorageBaseImpl_CopyTo(
    iface: *mut IStorage,
    ciidExclude: DWORD,
    rgiidExclude: *const IID,
    snbExclude: SNB,
    pstgDest: *mut IStorage,
) -> HRESULT {
    let This = impl_from_IStorage(iface);

    let mut skip_storage: BOOL = 0;
    let mut skip_stream: BOOL = 0;

    TRACE!(b"(%p, %d, %p, %p, %p)\n\0".as_ptr() as LPCSTR);

    if pstgDest.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    for i in 0..ciidExclude {
        if IsEqualGUID(&IID_IStorage, &*rgiidExclude.add(i as usize)) {
            skip_storage = 1;
        } else if IsEqualGUID(&IID_IStream, &*rgiidExclude.add(i as usize)) {
            skip_stream = 1;
        } else {
            WARN!(b"Unknown excluded GUID: %s\n\0".as_ptr() as LPCSTR);
        }
    }

    if skip_storage == 0 {
        let mut pstgDestAncestor = pstgDest;
        let mut pstgDestAncestorChild: *mut IStorage = core::ptr::null_mut();

        while pstgDestAncestor != iface {
            pstgDestAncestorChild = pstgDest;

            if (*pstgDestAncestor).lpVtbl == &TransactedSnapshotImpl_Vtbl as *const IStorageVtbl as *mut IStorageVtbl {
                let snapshot = pstgDestAncestor as *mut TransactedSnapshotImpl;
                pstgDestAncestor = &mut (*(*snapshot).transactedParent).IStorage_iface;
            } else if (*pstgDestAncestor).lpVtbl == &StorageInternalImpl_Vtbl as *const IStorageVtbl as *mut IStorageVtbl {
                let internal = pstgDestAncestor as *mut StorageInternalImpl;
                pstgDestAncestor = &mut (*(*internal).parentStorage).IStorage_iface;
            } else {
                break;
            }
        }

        if pstgDestAncestor == iface {
            let mut fail: BOOL = 1;

            if !pstgDestAncestorChild.is_null() && !snbExclude.is_null() {
                let ancestorChildBase = pstgDestAncestorChild as *mut StorageBaseImpl;
                let mut data: DirEntry = core::mem::zeroed();
                let mut snb = snbExclude;

                StorageBaseImpl_ReadDirEntry(
                    ancestorChildBase,
                    (*ancestorChildBase).storageDirEntry,
                    &mut data,
                );

                while !(*snb).is_null() && fail != 0 {
                    if wcscmp(data.name.as_ptr(), *snb) == 0 {
                        fail = 0;
                    }
                    snb = snb.add(1);
                }
            }

            if fail != 0 {
                return STG_E_ACCESSDENIED;
            }
        }
    }

    StorageBaseImpl_CopyStorageEntryTo(
        This,
        (*This).storageDirEntry,
        skip_storage,
        skip_stream,
        snbExclude,
        pstgDest,
    )
}
}

unsafe fn StorageImpl_ReadRawDirEntry(
    This: *mut StorageImpl,
    index: ULONG,
    buffer: *mut BYTE,
) -> HRESULT {
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut bytesRead: ULONG = 0;

    offset.QuadPart = index as ULONGLONG * RAW_DIRENTRY_SIZE as ULONGLONG;

    let hr = BlockChainStream_ReadAt(
        (*This).rootBlockChain,
        offset,
        RAW_DIRENTRY_SIZE,
        buffer as *mut core::ffi::c_void,
        &mut bytesRead,
    );

    if bytesRead != RAW_DIRENTRY_SIZE {
        return STG_E_READFAULT;
    }

    hr
}

unsafe extern "C" fn StorageImpl_CreateDirEntry(
    base: *mut StorageBaseImpl,
    newData: *const DirEntry,
    index: *mut DirRef,
) -> HRESULT {
    let storage = base as *mut StorageImpl;
    let mut currentEntryIndex: ULONG = 0;
    let mut newEntryIndex: ULONG = DIRENTRY_NULL;
    let mut hr: HRESULT;
    let mut currentData: [BYTE; RAW_DIRENTRY_SIZE as usize] = [0; RAW_DIRENTRY_SIZE as usize];
    let mut sizeOfNameString: WORD = 0;

    loop {
        hr = StorageImpl_ReadRawDirEntry(
            storage,
            currentEntryIndex,
            currentData.as_mut_ptr(),
        );

        if SUCCEEDED(hr) {
            StorageUtl_ReadWord(
                currentData.as_ptr(),
                OFFSET_PS_NAMELENGTH,
                &mut sizeOfNameString,
            );

            if sizeOfNameString == 0 {
                newEntryIndex = currentEntryIndex;
            }
        } else {
            newEntryIndex = currentEntryIndex;
        }
        currentEntryIndex += 1;

        if newEntryIndex != DIRENTRY_NULL {
            break;
        }
    }

    if FAILED(hr) {
        let mut emptyData: [BYTE; RAW_DIRENTRY_SIZE as usize] = [0; RAW_DIRENTRY_SIZE as usize];
        let mut newSize: ULARGE_INTEGER = core::mem::zeroed();
        let mut entryIndex: ULONG;
        let lastEntry: ULONG;
        let blockCount: ULONG;

        blockCount = BlockChainStream_GetCount((*storage).rootBlockChain) + 1;

        newSize.QuadPart = ((*storage).bigBlockSize as ULONGLONG) * (blockCount as ULONGLONG);

        BlockChainStream_SetSize((*storage).rootBlockChain, newSize);

        core::ptr::write_bytes(emptyData.as_mut_ptr(), 0, RAW_DIRENTRY_SIZE as usize);

        lastEntry = (*storage).bigBlockSize / RAW_DIRENTRY_SIZE * blockCount;

        entryIndex = newEntryIndex + 1;
        while entryIndex < lastEntry {
            StorageImpl_WriteRawDirEntry(
                storage,
                entryIndex,
                emptyData.as_ptr(),
            );
            entryIndex += 1;
        }

        StorageImpl_SaveFileHeader(storage);
    }

    UpdateRawDirEntry(currentData.as_mut_ptr(), newData);

    hr = StorageImpl_WriteRawDirEntry(storage, newEntryIndex, currentData.as_ptr());

    if SUCCEEDED(hr) {
        *index = newEntryIndex;
    }

    hr
}

unsafe fn StorageImpl_ReadDirEntry(
    This: *mut StorageImpl,
    index: DirRef,
    buffer: *mut DirEntry,
) -> HRESULT {
    let mut currentEntry: [BYTE; RAW_DIRENTRY_SIZE as usize] = [0; RAW_DIRENTRY_SIZE as usize];
    let readRes: HRESULT;

    readRes = StorageImpl_ReadRawDirEntry(This, index, currentEntry.as_mut_ptr());

    if SUCCEEDED(readRes) {
        core::ptr::write_bytes((*buffer).name.as_mut_ptr(), 0, (*buffer).name.len());
        core::ptr::copy_nonoverlapping(
            currentEntry.as_ptr().add(OFFSET_PS_NAME as usize) as *const WCHAR,
            (*buffer).name.as_mut_ptr(),
            DIRENTRY_NAME_BUFFER_LEN as usize,
        );
        TRACE!(b"storage name: %s\n\0".as_ptr() as PCHAR);

        core::ptr::copy_nonoverlapping(
            currentEntry.as_ptr().add(OFFSET_PS_STGTYPE as usize),
            core::ptr::addr_of_mut!((*buffer).stgType) as *mut BYTE,
            1,
        );

        StorageUtl_ReadWord(
            currentEntry.as_ptr(),
            OFFSET_PS_NAMELENGTH,
            core::ptr::addr_of_mut!((*buffer).sizeOfNameString),
        );

        StorageUtl_ReadDWord(
            currentEntry.as_ptr(),
            OFFSET_PS_LEFTCHILD,
            core::ptr::addr_of_mut!((*buffer).leftChild),
        );

        StorageUtl_ReadDWord(
            currentEntry.as_ptr(),
            OFFSET_PS_RIGHTCHILD,
            core::ptr::addr_of_mut!((*buffer).rightChild),
        );

        StorageUtl_ReadDWord(
            currentEntry.as_ptr(),
            OFFSET_PS_DIRROOT,
            core::ptr::addr_of_mut!((*buffer).dirRootEntry),
        );

        StorageUtl_ReadGUID(
            currentEntry.as_ptr(),
            OFFSET_PS_GUID,
            core::ptr::addr_of_mut!((*buffer).clsid),
        );

        StorageUtl_ReadDWord(
            currentEntry.as_ptr(),
            OFFSET_PS_CTIMELOW,
            core::ptr::addr_of_mut!((*buffer).ctime.dwLowDateTime),
        );

        StorageUtl_ReadDWord(
            currentEntry.as_ptr(),
            OFFSET_PS_CTIMEHIGH,
            core::ptr::addr_of_mut!((*buffer).ctime.dwHighDateTime),
        );

        StorageUtl_ReadDWord(
            currentEntry.as_ptr(),
            OFFSET_PS_MTIMELOW,
            core::ptr::addr_of_mut!((*buffer).mtime.dwLowDateTime),
        );

        StorageUtl_ReadDWord(
            currentEntry.as_ptr(),
            OFFSET_PS_MTIMEHIGH,
            core::ptr::addr_of_mut!((*buffer).mtime.dwHighDateTime),
        );

        StorageUtl_ReadDWord(
            currentEntry.as_ptr(),
            OFFSET_PS_STARTBLOCK,
            core::ptr::addr_of_mut!((*buffer).startingBlock),
        );

        StorageUtl_ReadDWord(
            currentEntry.as_ptr(),
            OFFSET_PS_SIZE,
            core::ptr::addr_of_mut!((*buffer).size.u.LowPart),
        );

        if (*This).bigBlockSize < 4096 {
            (*buffer).size.u.HighPart = 0;
        } else {
            StorageUtl_ReadDWord(
                currentEntry.as_ptr(),
                OFFSET_PS_SIZE_HIGH,
                core::ptr::addr_of_mut!((*buffer).size.u.HighPart),
            );
        }
    }

    readRes
}

unsafe fn Storage32Impl_SmallBlocksToBigBlocks(
    This: *mut StorageImpl,
    ppsbChain: *mut *mut SmallBlockChainStream,
) -> *mut BlockChainStream {
    let mut bbHeadOfChain: ULONG = BLOCK_END_OF_CHAIN;
    let mut size: ULARGE_INTEGER;
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut cbRead: ULONG = 0;
    let mut cbWritten: ULONG = 0;
    let mut cbTotalRead: ULARGE_INTEGER = core::mem::zeroed();
    let streamEntryRef: DirRef;
    let mut resWrite: HRESULT = S_OK;
    let mut resRead: HRESULT;
    let mut streamEntry: DirEntry = core::mem::zeroed();
    let buffer: *mut BYTE;
    let bbTempChain: *mut BlockChainStream;
    let bigBlockChain: *mut BlockChainStream;

    bbTempChain = BlockChainStream_Construct(
        This,
        &mut bbHeadOfChain,
        DIRENTRY_NULL,
    );
    if bbTempChain.is_null() {
        return core::ptr::null_mut();
    }

    size = SmallBlockChainStream_GetSize(*ppsbChain);
    BlockChainStream_SetSize(bbTempChain, size);

    offset.u.LowPart = 0;
    offset.u.HighPart = 0;
    cbTotalRead.QuadPart = 0;

    buffer = HeapAlloc(GetProcessHeap(), 0, (DEF_SMALL_BLOCK_SIZE as usize) as SIZE_T) as *mut BYTE;
    
    let final_resRead = loop {
        resRead = SmallBlockChainStream_ReadAt(
            *ppsbChain,
            offset,
            min((*This).smallBlockSize, size.u.LowPart - offset.u.LowPart),
            buffer as *mut core::ffi::c_void,
            &mut cbRead,
        );
        if FAILED(resRead) {
            break resRead;
        }

        if cbRead > 0 {
            cbTotalRead.QuadPart += cbRead as u64;

            resWrite = BlockChainStream_WriteAt(
                bbTempChain,
                offset,
                cbRead,
                buffer as *const core::ffi::c_void,
                &mut cbWritten,
            );

            if FAILED(resWrite) {
                break resRead;
            }

            offset.u.LowPart += cbRead;
        } else {
            break STG_E_READFAULT;
        }

        if cbTotalRead.QuadPart >= size.QuadPart {
            break resRead;
        }
    };
    
    HeapFree(GetProcessHeap(), 0, buffer as *mut core::ffi::c_void);

    size.u.HighPart = 0;
    size.u.LowPart = 0;

    if FAILED(final_resRead) || FAILED(resWrite) {
        ERR(b"conversion failed: resRead = 0x%08x, resWrite = 0x%08x\n\0".as_ptr() as PCHAR);
        BlockChainStream_SetSize(bbTempChain, size);
        BlockChainStream_Destroy(bbTempChain);
        return core::ptr::null_mut();
    }

    streamEntryRef = (**ppsbChain).ownerDirEntry;
    SmallBlockChainStream_SetSize(*ppsbChain, size);
    SmallBlockChainStream_Destroy(*ppsbChain);
    *ppsbChain = core::ptr::null_mut();

    StorageImpl_ReadDirEntry(This, streamEntryRef, &mut streamEntry);

    streamEntry.startingBlock = bbHeadOfChain;

    StorageImpl_WriteDirEntry(This, streamEntryRef, &streamEntry);

    BlockChainStream_Destroy(bbTempChain);
    bigBlockChain = BlockChainStream_Construct(
        This,
        core::ptr::null_mut(),
        streamEntryRef,
    );

    bigBlockChain
}

unsafe fn Storage32Impl_BigBlocksToSmallBlocks(
    This: *mut StorageImpl,
    ppbbChain: *mut *mut BlockChainStream,
    newSize: ULARGE_INTEGER,
) -> *mut SmallBlockChainStream {
    let mut size: ULARGE_INTEGER;
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut cbTotalRead: ULARGE_INTEGER = core::mem::zeroed();
    let mut cbRead: ULONG = 0;
    let mut cbWritten: ULONG = 0;
    let mut sbHeadOfChain: ULONG = BLOCK_END_OF_CHAIN;
    let streamEntryRef: DirRef;
    let mut resWrite: HRESULT = S_OK;
    let mut resRead: HRESULT = S_OK;
    let mut streamEntry: DirEntry = core::mem::zeroed();
    let buffer: *mut BYTE;
    let sbTempChain: *mut SmallBlockChainStream;

    TRACE!(b"%p %p\n\0".as_ptr() as PCHAR);

    sbTempChain = SmallBlockChainStream_Construct(
        This,
        &mut sbHeadOfChain,
        DIRENTRY_NULL,
    );

    if sbTempChain.is_null() {
        return core::ptr::null_mut();
    }

    SmallBlockChainStream_SetSize(sbTempChain, newSize);
    size = BlockChainStream_GetSize(*ppbbChain);
    size.QuadPart = min(size.QuadPart, newSize.QuadPart);

    offset.u.HighPart = 0;
    offset.u.LowPart = 0;
    cbTotalRead.QuadPart = 0;
    buffer = HeapAlloc(GetProcessHeap(), 0, ((*This).bigBlockSize as usize) as SIZE_T) as *mut BYTE;

    while cbTotalRead.QuadPart < size.QuadPart {
        resRead = BlockChainStream_ReadAt(
            *ppbbChain,
            offset,
            min(
                (*This).bigBlockSize,
                size.u.LowPart - offset.u.LowPart,
            ),
            buffer as *mut core::ffi::c_void,
            &mut cbRead,
        );

        if FAILED(resRead) {
            break;
        }

        if cbRead > 0 {
            cbTotalRead.QuadPart += cbRead as ULONGLONG;

            resWrite = SmallBlockChainStream_WriteAt(
                sbTempChain,
                offset,
                cbRead,
                buffer as *const core::ffi::c_void,
                &mut cbWritten,
            );

            if FAILED(resWrite) {
                break;
            }

            offset.u.LowPart += cbRead;
        } else {
            resRead = STG_E_READFAULT;
            break;
        }
    }

    HeapFree(GetProcessHeap(), 0, buffer as *mut core::ffi::c_void);

    size.u.HighPart = 0;
    size.u.LowPart = 0;

    if FAILED(resRead) || FAILED(resWrite) {
        ERR(b"conversion failed: resRead = 0x%08x, resWrite = 0x%08x\n\0".as_ptr() as PCHAR);
        SmallBlockChainStream_SetSize(sbTempChain, size);
        SmallBlockChainStream_Destroy(sbTempChain);
        return core::ptr::null_mut();
    }

    streamEntryRef = (**ppbbChain).ownerDirEntry;
    BlockChainStream_SetSize(*ppbbChain, size);
    BlockChainStream_Destroy(*ppbbChain);
    *ppbbChain = core::ptr::null_mut();

    StorageImpl_ReadDirEntry(This, streamEntryRef, &mut streamEntry);
    streamEntry.startingBlock = sbHeadOfChain;
    StorageImpl_WriteDirEntry(This, streamEntryRef, &streamEntry);

    SmallBlockChainStream_Destroy(sbTempChain);
    SmallBlockChainStream_Construct(This, core::ptr::null_mut(), streamEntryRef)
}

stdmethod! {
fn StorageBaseImpl_SetElementTimes(
    iface: *mut IStorage,
    pwcsName: *const OLECHAR,
    pctime: *const FILETIME,
    patime: *const FILETIME,
    pmtime: *const FILETIME,
) -> HRESULT {
    
    FIXME(b"(%s,...), stub!\n\0".as_ptr() as *const i8);
    S_OK
}
}

unsafe extern "C" fn StorageImpl_BaseReadDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
    data: *mut DirEntry,
) -> HRESULT {
    let This = base as *mut StorageImpl;
    StorageImpl_ReadDirEntry(This, index, data)
}

unsafe extern "C" fn StorageImpl_StreamReadAt(
    base: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *mut core::ffi::c_void,
    bytesRead: *mut ULONG,
) -> HRESULT {
    let This = base as *mut StorageImpl;
    let mut data: DirEntry = core::mem::zeroed();
    let hr: HRESULT;
    let bytesToRead: ULONG;

    hr = StorageImpl_ReadDirEntry(This, index, core::ptr::addr_of_mut!(data));
    if FAILED(hr) {
        return hr;
    }

    if (*core::ptr::addr_of!(data.size)).QuadPart == 0 {
        *bytesRead = 0;
        return S_OK;
    }

    if offset.QuadPart + size as u64 > (*core::ptr::addr_of!(data.size)).QuadPart {
        bytesToRead = ((*core::ptr::addr_of!(data.size)).QuadPart - offset.QuadPart) as ULONG;
    } else {
        bytesToRead = size;
    }

    if (*core::ptr::addr_of!(data.size)).QuadPart < LIMIT_TO_USE_SMALL_BLOCK as u64 {
        let stream: *mut SmallBlockChainStream;

        stream = SmallBlockChainStream_Construct(This, core::ptr::null_mut(), index);
        if stream.is_null() {
            return E_OUTOFMEMORY;
        }

        let hr = SmallBlockChainStream_ReadAt(stream, offset, bytesToRead, buffer, bytesRead);

        SmallBlockChainStream_Destroy(stream);

        return hr;
    } else {
        let stream: *mut BlockChainStream;

        stream = *StorageImpl_GetCachedBlockChainStream(This, index);
        if stream.is_null() {
            return E_OUTOFMEMORY;
        }

        let hr = BlockChainStream_ReadAt(stream, offset, bytesToRead, buffer, bytesRead);

        return hr;
    }
}

unsafe extern "C" fn StorageImpl_StreamSetSize(
    base: *mut StorageBaseImpl,
    index: DirRef,
    newsize: ULARGE_INTEGER,
) -> HRESULT {
    let mut newsize = newsize;
    let This = base as *mut StorageImpl;
    let mut data: DirEntry = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut smallblock: *mut SmallBlockChainStream = core::ptr::null_mut();
    let mut pbigblock: *mut *mut BlockChainStream = core::ptr::null_mut();
    let mut bigblock: *mut BlockChainStream = core::ptr::null_mut();

    hr = StorageImpl_ReadDirEntry(This, index, &mut data);
    if FAILED(hr) {
        return hr;
    }

    if ((*This).base.openFlags & STGM_SIMPLE) != 0 {
        let max_size = max(newsize.QuadPart, LIMIT_TO_USE_SMALL_BLOCK as ULONGLONG);
        newsize.QuadPart = max_size;
    }

    if data.size.QuadPart == newsize.QuadPart {
        return S_OK;
    }

    if data.size.QuadPart == 0 {
        if newsize.QuadPart < LIMIT_TO_USE_SMALL_BLOCK as ULONGLONG {
            smallblock = SmallBlockChainStream_Construct(This, core::ptr::null_mut(), index);
            if smallblock.is_null() {
                return E_OUTOFMEMORY;
            }
        } else {
            pbigblock = StorageImpl_GetCachedBlockChainStream(This, index);
            bigblock = *pbigblock;
            if bigblock.is_null() {
                return E_OUTOFMEMORY;
            }
        }
    } else if data.size.QuadPart < LIMIT_TO_USE_SMALL_BLOCK as ULONGLONG {
        smallblock = SmallBlockChainStream_Construct(This, core::ptr::null_mut(), index);
        if smallblock.is_null() {
            return E_OUTOFMEMORY;
        }
    } else {
        pbigblock = StorageImpl_GetCachedBlockChainStream(This, index);
        bigblock = *pbigblock;
        if bigblock.is_null() {
            return E_OUTOFMEMORY;
        }
    }

    if !smallblock.is_null() && newsize.QuadPart >= LIMIT_TO_USE_SMALL_BLOCK as ULONGLONG {
        bigblock = Storage32Impl_SmallBlocksToBigBlocks(This, &mut smallblock);
        if bigblock.is_null() {
            SmallBlockChainStream_Destroy(smallblock);
            return E_FAIL;
        }

        pbigblock = StorageImpl_GetFreeBlockChainCacheEntry(This);
        *pbigblock = bigblock;
    } else if !bigblock.is_null() && newsize.QuadPart < LIMIT_TO_USE_SMALL_BLOCK as ULONGLONG {
        smallblock = Storage32Impl_BigBlocksToSmallBlocks(This, pbigblock, newsize);
        if smallblock.is_null() {
            return E_FAIL;
        }
    }

    if !smallblock.is_null() {
        SmallBlockChainStream_SetSize(smallblock, newsize);
        SmallBlockChainStream_Destroy(smallblock);
    } else {
        BlockChainStream_SetSize(bigblock, newsize);
    }

    hr = StorageImpl_ReadDirEntry(This, index, &mut data);
    if SUCCEEDED(hr) {
        data.size = newsize;

        hr = StorageImpl_WriteDirEntry(This, index, &data);
    }
    hr
}

unsafe extern "C" fn StorageImpl_StreamWriteAt(
    base: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *const core::ffi::c_void,
    bytesWritten: *mut ULONG,
) -> HRESULT {
    let This = base as *mut StorageImpl;
    let mut data: DirEntry = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut newSize: ULARGE_INTEGER = core::mem::zeroed();

    hr = StorageImpl_ReadDirEntry(This, index, &mut data);
    if FAILED(hr) {
        return hr;
    }

    newSize.QuadPart = offset.QuadPart + size as u64;

    if newSize.QuadPart > data.size.QuadPart {
        hr = StorageImpl_StreamSetSize(base, index, newSize);
        if FAILED(hr) {
            return hr;
        }

        hr = StorageImpl_ReadDirEntry(This, index, &mut data);
        if FAILED(hr) {
            return hr;
        }
    }

    if data.size.QuadPart < LIMIT_TO_USE_SMALL_BLOCK as ULONGLONG {
        let stream: *mut SmallBlockChainStream;

        stream = SmallBlockChainStream_Construct(This, core::ptr::null_mut(), index);
        if stream.is_null() {
            return E_OUTOFMEMORY;
        }

        hr = SmallBlockChainStream_WriteAt(stream, offset, size, buffer, bytesWritten);

        SmallBlockChainStream_Destroy(stream);

        return hr;
    } else {
        let stream: *mut BlockChainStream;

        stream = *StorageImpl_GetCachedBlockChainStream(This, index);
        if stream.is_null() {
            return E_OUTOFMEMORY;
        }

        return BlockChainStream_WriteAt(stream, offset, size, buffer, bytesWritten);
    }
}

unsafe extern "C" fn StorageImpl_StreamLink(base: *mut StorageBaseImpl, dst: DirRef, src: DirRef) -> HRESULT {
    let This = base as *mut StorageImpl;
    let mut dst_data: DirEntry = core::mem::zeroed();
    let mut src_data: DirEntry = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = StorageImpl_ReadDirEntry(This, dst, &mut dst_data);

    if SUCCEEDED(hr) {
        hr = StorageImpl_ReadDirEntry(This, src, &mut src_data);
    }

    if SUCCEEDED(hr) {
        StorageImpl_DeleteCachedBlockChainStream(This, src);
        dst_data.startingBlock = src_data.startingBlock;
        dst_data.size = src_data.size;

        hr = StorageImpl_WriteDirEntry(This, dst, &dst_data);
    }

    hr
}

unsafe fn StorageImpl_Refresh(This: *mut StorageImpl, new_object: BOOL, create: BOOL) -> HRESULT {
    let mut hr: HRESULT;
    let mut currentEntry: DirEntry = core::mem::zeroed();
    let mut currentEntryRef: DirRef;
    let mut blockChainStream: *mut BlockChainStream;

    if create != 0 {
        let mut size: ULARGE_INTEGER = core::mem::zeroed();
        let mut bigBlockBuffer: [BYTE; MAX_BIG_BLOCK_SIZE as usize] = [0; MAX_BIG_BLOCK_SIZE as usize];

        size.QuadPart = 0;
        ILockBytes_SetSize((*This).lockBytes, size);

        // C: memset(This->bigBlockDepotStart, BLOCK_UNUSED, sizeof(This->bigBlockDepotStart)).
        // `write_bytes` counts ELEMENTS, not bytes: with a *mut ULONG this wrote
        // 109*4 = 436 ULONGs = 1744 bytes — a 4x overrun straight through transactionSig and
        // the depot caches. The pointer must be *mut u8 for the count to mean bytes.
        core::ptr::write_bytes(
            (*This).bigBlockDepotStart.as_mut_ptr() as *mut u8,
            BLOCK_UNUSED as u8,
            core::mem::size_of_val(&(*This).bigBlockDepotStart),
        );

        (*This).bigBlockDepotCount = 1;
        (*This).bigBlockDepotStart[0] = 0;
        (*This).rootStartBlock = 1;
        (*This).smallBlockLimit = LIMIT_TO_USE_SMALL_BLOCK;
        (*This).smallBlockDepotStart = BLOCK_END_OF_CHAIN;
        if (*This).bigBlockSize == 4096 {
            (*This).bigBlockSizeBits = MAX_BIG_BLOCK_SIZE_BITS;
        } else {
            (*This).bigBlockSizeBits = MIN_BIG_BLOCK_SIZE_BITS;
        }
        (*This).smallBlockSizeBits = DEF_SMALL_BLOCK_SIZE_BITS;
        (*This).extBigBlockDepotStart = BLOCK_END_OF_CHAIN;
        (*This).extBigBlockDepotCount = 0;

        StorageImpl_SaveFileHeader(This);

        size.u.HighPart = 0;
        size.u.LowPart = (*This).bigBlockSize * 3;
        ILockBytes_SetSize((*This).lockBytes, size);

        core::ptr::write_bytes(bigBlockBuffer.as_mut_ptr(), BLOCK_UNUSED as u8, (*This).bigBlockSize as usize);
        StorageUtl_WriteDWord(bigBlockBuffer.as_mut_ptr(), 0, BLOCK_SPECIAL);
        StorageUtl_WriteDWord(bigBlockBuffer.as_mut_ptr(), core::mem::size_of::<ULONG>() as ULONG, BLOCK_END_OF_CHAIN);
        StorageImpl_WriteBigBlock(This, 0, bigBlockBuffer.as_ptr() as *const core::ffi::c_void);
    } else {
        hr = StorageImpl_LoadFileHeader(This);

        if FAILED(hr) {
            return hr;
        }
    }

    (*This).indexBlockDepotCached = 0xFFFFFFFF;
    (*This).indexExtBlockDepotCached = 0xFFFFFFFF;

    (*This).prevFreeBlock = 0;

    (*This).firstFreeSmallBlock = 0;

    if (*This).extBigBlockDepotCount != 0 {
        let mut current_block: ULONG = (*This).extBigBlockDepotStart;
        let cache_size: ULONG = (*This).extBigBlockDepotCount * 2;
        let mut i: ULONG;

        (*This).extBigBlockDepotLocations = HeapAlloc(GetProcessHeap(), 0, ((core::mem::size_of::<ULONG>() * cache_size as usize) as usize) as SIZE_T) as *mut ULONG;
        if (*This).extBigBlockDepotLocations.is_null() {
            return E_OUTOFMEMORY;
        }

        (*This).extBigBlockDepotLocationsSize = cache_size;

        i = 0;
        while i < (*This).extBigBlockDepotCount {
            if current_block == BLOCK_END_OF_CHAIN {
                WARN!(b"File has too few extended big block depot blocks.\n\0".as_ptr() as PCHAR);
                return STG_E_DOCFILECORRUPT;
            }
            *(*This).extBigBlockDepotLocations.add(i as usize) = current_block;
            current_block = Storage32Impl_GetNextExtendedBlock(This, current_block);
            i += 1;
        }
    } else {
        (*This).extBigBlockDepotLocations = core::ptr::null_mut();
        (*This).extBigBlockDepotLocationsSize = 0;
    }

    blockChainStream = BlockChainStream_Construct(This, core::ptr::addr_of_mut!((*This).rootStartBlock), DIRENTRY_NULL);
    if blockChainStream.is_null() {
        return STG_E_READFAULT;
    }
    if new_object == 0 {
        BlockChainStream_Destroy((*This).rootBlockChain);
    }
    (*This).rootBlockChain = blockChainStream;

    blockChainStream = BlockChainStream_Construct(This, core::ptr::addr_of_mut!((*This).smallBlockDepotStart), DIRENTRY_NULL);
    if blockChainStream.is_null() {
        return STG_E_READFAULT;
    }
    if new_object == 0 {
        BlockChainStream_Destroy((*This).smallBlockDepotChain);
    }
    (*This).smallBlockDepotChain = blockChainStream;

    if create != 0 {
        static rootentryW: [u16; 11] = [82, 111, 111, 116, 32, 69, 110, 116, 114, 121, 0];
        let mut rootEntry: DirEntry = core::mem::zeroed();

        core::ptr::write_bytes(&mut rootEntry as *mut DirEntry as *mut u8, 0, core::mem::size_of::<DirEntry>());
        lstrcpyW(rootEntry.name.as_mut_ptr(), rootentryW.as_ptr());
        rootEntry.sizeOfNameString = core::mem::size_of_val(&rootentryW) as WORD;
        rootEntry.stgType = STGTY_ROOT as BYTE;
        rootEntry.leftChild = DIRENTRY_NULL;
        rootEntry.rightChild = DIRENTRY_NULL;
        rootEntry.dirRootEntry = DIRENTRY_NULL;
        rootEntry.startingBlock = BLOCK_END_OF_CHAIN;
        rootEntry.size.u.HighPart = 0;
        rootEntry.size.u.LowPart = 0;

        StorageImpl_WriteDirEntry(This, 0, &rootEntry);
    }

    currentEntryRef = 0;

    loop {
        hr = StorageImpl_ReadDirEntry(This, currentEntryRef, &mut currentEntry);

        if SUCCEEDED(hr) {
            if currentEntry.sizeOfNameString != 0 && currentEntry.stgType as DWORD == STGTY_ROOT {
                (*This).base.storageDirEntry = currentEntryRef;
            }
        }

        currentEntryRef += 1;

        if !(SUCCEEDED(hr) && (*This).base.storageDirEntry == DIRENTRY_NULL) {
            break;
        }
    }

    if FAILED(hr) {
        return STG_E_READFAULT;
    }

    blockChainStream = BlockChainStream_Construct(This, core::ptr::null_mut(), (*This).base.storageDirEntry);
    if blockChainStream.is_null() {
        return STG_E_READFAULT;
    }
    if new_object == 0 {
        BlockChainStream_Destroy((*This).smallBlockRootChain);
    }
    (*This).smallBlockRootChain = blockChainStream;

    if new_object == 0 {
        let mut i: i32 = 0;
        while i < BLOCKCHAIN_CACHE_SIZE as i32 {
            BlockChainStream_Destroy((*This).blockChainCache[i as usize]);
            (*This).blockChainCache[i as usize] = core::ptr::null_mut();
            i += 1;
        }
    }

    hr
}

unsafe extern "C" fn StorageImpl_GetTransactionSig(
    base: *mut StorageBaseImpl,
    result: *mut ULONG,
    refresh: BOOL,
) -> HRESULT {
    let This = base as *mut StorageImpl;
    let mut hr: HRESULT = S_OK;
    let oldTransactionSig = (*This).transactionSig;

    if refresh != 0 {
        let mut offset: ULARGE_INTEGER = core::mem::zeroed();
        let mut bytes_read: ULONG = 0;
        let mut data: [BYTE; 4] = [0; 4];

        offset.u.HighPart = 0;
        offset.u.LowPart = OFFSET_TRANSACTIONSIG;
        hr = StorageImpl_ReadAt(This, offset, data.as_mut_ptr() as *mut core::ffi::c_void, 4, &mut bytes_read);

        if SUCCEEDED(hr) {
            StorageUtl_ReadDWord(data.as_ptr(), 0, &mut (*This).transactionSig);

            if oldTransactionSig != (*This).transactionSig {
                TRACE!(b"signature changed\n\0".as_ptr() as PCHAR);

                hr = StorageImpl_Refresh(This, 0, 0);
            }

            if FAILED(hr) {
                (*This).transactionSig = oldTransactionSig;
            }
        }
    }

    *result = (*This).transactionSig;

    hr
}

unsafe fn StorageImpl_Construct(
    hFile: HANDLE,
    pwcsName: LPCOLESTR,
    pLkbyt: *mut ILockBytes,
    openFlags: DWORD,
    _fileBased: BOOL,
    create: BOOL,
    sector_size: ULONG,
    result: *mut *mut StorageImpl,
) -> HRESULT {
    

    let mut hr: HRESULT = S_OK;
    let mut stat: STATSTG = core::mem::zeroed();

    if FAILED(validateSTGM(openFlags)) {
        return STG_E_INVALIDFLAG;
    }

    let This = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<StorageImpl>()) as SIZE_T) as *mut StorageImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    memset(This as *mut core::ffi::c_void, 0, core::mem::size_of::<StorageImpl>());

    list_init(core::ptr::addr_of_mut!((*This).base.strmHead));
    list_init(core::ptr::addr_of_mut!((*This).base.storageHead));

    (*This).base.IStorage_iface.lpVtbl = &StorageImpl_Vtbl as *const IStorageVtbl as *mut IStorageVtbl;
    (*This).base.IPropertySetStorage_iface.lpVtbl = &IPropertySetStorage_Vtbl as *const IPropertySetStorageVtbl as *mut IPropertySetStorageVtbl;
    (*This).base.IDirectWriterLock_iface.lpVtbl = &DirectWriterLockVtbl as *const IDirectWriterLockVtbl as *mut IDirectWriterLockVtbl;
    (*This).base.baseVtbl = &StorageImpl_BaseVtbl;
    (*This).base.openFlags = openFlags & !STGM_CREATE;
    (*This).base.ref_ = 1;
    (*This).base.create = create;

    if openFlags == (STGM_DIRECT_SWMR | STGM_READWRITE | STGM_SHARE_DENY_WRITE) {
        (*This).base.lockingrole = SWMR_Writer;
    } else if openFlags == (STGM_DIRECT_SWMR | STGM_READ | STGM_SHARE_DENY_NONE) {
        (*This).base.lockingrole = SWMR_Reader;
    } else {
        (*This).base.lockingrole = SWMR_None;
    }

    (*This).base.reverted = 0;

    (*This).bigBlockSize = sector_size;
    (*This).smallBlockSize = DEF_SMALL_BLOCK_SIZE as ULONG;

    if !hFile.is_null() {
        hr = FileLockBytesImpl_Construct(hFile, openFlags, pwcsName, core::ptr::addr_of_mut!((*This).lockBytes));
    } else {
        (*This).lockBytes = pLkbyt;
        ILockBytes_AddRef(pLkbyt);
    }

    if SUCCEEDED(hr) {
        hr = ILockBytes_Stat((*This).lockBytes, core::ptr::addr_of_mut!(stat), STATFLAG_NONAME);
    }

    if SUCCEEDED(hr) {
        (*This).locks_supported = stat.grfLocksSupported;
        if hFile.is_null() {
            (*This).locks_supported &= !WINE_LOCK_READ;
        }

        hr = StorageImpl_GrabLocks(This, openFlags);
    }

    if SUCCEEDED(hr) {
        hr = StorageImpl_Refresh(This, 1, create);
    }

    if FAILED(hr) {
        IStorage_Release(core::ptr::addr_of_mut!((*This).base.IStorage_iface) as *mut IStorage);
        *result = core::ptr::null_mut();
    } else {
        StorageImpl_Flush(&mut (*This).base);
        *result = This;
    }

    hr
}

unsafe fn TransactedSnapshotImpl_Construct(
    parentStorage: *mut StorageBaseImpl,
    result: *mut *mut TransactedSnapshotImpl,
) -> HRESULT {
    let mut hr: HRESULT;

    *result = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<TransactedSnapshotImpl>()) as SIZE_T) as *mut TransactedSnapshotImpl;
    if !(*result).is_null() {
        let mut scratch: *mut IStorage = core::ptr::null_mut();

        (*(*result)).base.IStorage_iface.lpVtbl = &TransactedSnapshotImpl_Vtbl as *const IStorageVtbl as *mut IStorageVtbl;

        (*(*result)).base.IPropertySetStorage_iface.lpVtbl = (*parentStorage).IPropertySetStorage_iface.lpVtbl;
        (*(*result)).base.baseVtbl = &TransactedSnapshotImpl_BaseVtbl;

        list_init(core::ptr::addr_of_mut!((*(*result)).base.strmHead));

        list_init(core::ptr::addr_of_mut!((*(*result)).base.storageHead));

        (*(*result)).base.ref_ = 1;

        (*(*result)).base.openFlags = (*parentStorage).openFlags;

        StorageBaseImpl_GetTransactionSig(parentStorage, core::ptr::addr_of_mut!((*(*result)).lastTransactionSig), 0);

        hr = StgCreateDocfile(
            core::ptr::null(),
            STGM_READWRITE | STGM_SHARE_EXCLUSIVE | STGM_CREATE | STGM_DELETEONRELEASE,
            0,
            core::ptr::addr_of_mut!(scratch),
        );
        (*(*result)).scratch = impl_from_IStorage(scratch);

        if SUCCEEDED(hr) {
            let num_entries: ULONG = 20;

            (*(*result)).entries = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<TransactedDirEntry>() * num_entries as usize) as SIZE_T,
            ) as *mut TransactedDirEntry;
            (*(*result)).entries_size = num_entries;
            (*(*result)).firstFreeEntry = 0;

            if !(*(*result)).entries.is_null() {
                (*(*result)).transactedParent = parentStorage;

                (*parentStorage).transactedChild = &mut (*(*result)).base;

                (*(*result)).base.storageDirEntry = TransactedSnapshotImpl_CreateStubEntry(*result, (*parentStorage).storageDirEntry);
            } else {
                IStorage_Release(scratch);

                hr = 0x8007000Eu32 as HRESULT;
            }
        }

        if FAILED(hr) {
            HeapFree(GetProcessHeap(), 0, *result as LPVOID);
        }

        return hr;
    } else {
        return 0x8007000Eu32 as HRESULT;
    }
}

unsafe fn TransactedSharedImpl_Construct(
    parentStorage: *mut StorageBaseImpl,
    result: *mut *mut TransactedSharedImpl,
) -> HRESULT {
    let mut hr: HRESULT;

    *result = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<TransactedSharedImpl>()) as SIZE_T) as *mut TransactedSharedImpl;
    if !(*result).is_null() {
        let mut scratch: *mut IStorage = core::ptr::null_mut();

        (**result).base.IStorage_iface.lpVtbl = &TransactedSharedImpl_Vtbl as *const IStorageVtbl as *mut IStorageVtbl;

        (**result).base.IPropertySetStorage_iface.lpVtbl = (*parentStorage).IPropertySetStorage_iface.lpVtbl;
        (**result).base.baseVtbl = &TransactedSharedImpl_BaseVtbl;

        list_init(core::ptr::addr_of_mut!((**result).base.strmHead));

        list_init(core::ptr::addr_of_mut!((**result).base.storageHead));

        (**result).base.ref_ = 1;

        (**result).base.openFlags = (*parentStorage).openFlags;

        hr = StorageBaseImpl_LockTransaction(parentStorage, 0);

        if SUCCEEDED(hr) {
            let mut stgo: STGOPTIONS = core::mem::zeroed();

            StorageBaseImpl_GetTransactionSig(parentStorage, core::ptr::addr_of_mut!((**result).lastTransactionSig), 0);

            stgo.usVersion = 1;
            stgo.reserved = 0;
            stgo.ulSectorSize = 4096;
            stgo.pwcsTemplateFile = core::ptr::null_mut();

            hr = StgCreateStorageEx(
                core::ptr::null(),
                STGM_READWRITE | STGM_SHARE_EXCLUSIVE | STGM_CREATE | STGM_DELETEONRELEASE | STGM_TRANSACTED,
                STGFMT_DOCFILE,
                0,
                core::ptr::addr_of_mut!(stgo),
                core::ptr::null_mut(),
                &IID_IStorage,
                core::ptr::addr_of_mut!(scratch) as *mut *mut core::ffi::c_void,
            );
            (**result).scratch = impl_from_IStorage(scratch) as *mut TransactedSnapshotImpl;

            if SUCCEEDED(hr) {
                hr = StorageBaseImpl_CopyStorageTree(
                    core::ptr::addr_of_mut!((*(**result).scratch).base),
                    (*(**result).scratch).base.storageDirEntry,
                    parentStorage,
                    (*parentStorage).storageDirEntry,
                );

                if SUCCEEDED(hr) {
                    hr = IStorage_Commit(scratch, STGC_DEFAULT);

                    (**result).base.storageDirEntry = (*(**result).scratch).base.storageDirEntry;
                    (**result).transactedParent = parentStorage;
                }

                if FAILED(hr) {
                    IStorage_Release(scratch);
                }
            }

            StorageBaseImpl_UnlockTransaction(parentStorage, 0);
        }

        if FAILED(hr) {
            HeapFree(GetProcessHeap(), 0, *result as LPVOID);
        }

        return hr;
    } else {
        return E_OUTOFMEMORY;
    }
}

pub unsafe fn StorageBaseImpl_AddStream(stg: *mut StorageBaseImpl, strm: *mut StgStreamImpl) {
    TRACE!(b"Stream added (stg=%p strm=%p)\n\0".as_ptr() as PCHAR);
    list_add_tail(
        core::ptr::addr_of_mut!((*stg).strmHead),
        core::ptr::addr_of_mut!((*strm).StrmListEntry)
    );
}

unsafe fn Storage_ConstructTransacted(
    parentStorage: *mut StorageBaseImpl,
    toplevel: BOOL,
    result: *mut *mut StorageBaseImpl,
) -> HRESULT {
    static mut fixme_flags: DWORD = STGM_NOSCRATCH | STGM_NOSNAPSHOT;

    if ((*parentStorage).openFlags & fixme_flags) != 0 {
        fixme_flags &= !((*parentStorage).openFlags);
        FIXME(b"Unimplemented flags %x\n\0".as_ptr() as PCHAR);
    }

    if toplevel != 0 
        && ((*parentStorage).openFlags & STGM_NOSNAPSHOT) == 0
        && STGM_SHARE_MODE((*parentStorage).openFlags) != STGM_SHARE_DENY_WRITE
        && STGM_SHARE_MODE((*parentStorage).openFlags) != STGM_SHARE_EXCLUSIVE
    {
        return TransactedSharedImpl_Construct(
            parentStorage,
            result as *mut *mut TransactedSharedImpl,
        );
    }

    TransactedSnapshotImpl_Construct(
        parentStorage,
        result as *mut *mut TransactedSnapshotImpl,
    )
}

unsafe fn Storage_Construct(
    hFile: HANDLE,
    pwcsName: LPCOLESTR,
    pLkbyt: *mut ILockBytes,
    openFlags: DWORD,
    fileBased: BOOL,
    create: BOOL,
    sector_size: ULONG,
    result: *mut *mut StorageBaseImpl,
) -> HRESULT {
    let mut newStorage: *mut StorageImpl = core::ptr::null_mut();
    let mut newTransactedStorage: *mut StorageBaseImpl = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = StorageImpl_Construct(
        hFile,
        pwcsName,
        pLkbyt,
        openFlags,
        fileBased,
        create,
        sector_size,
        &mut newStorage,
    );
    if FAILED(hr) {
        return hr;
    }

    if (openFlags & STGM_TRANSACTED) != 0 {
        hr = Storage_ConstructTransacted(
            &mut (*newStorage).base,
            1,
            &mut newTransactedStorage,
        );
        if FAILED(hr) {
            IStorage_Release(&mut (*newStorage).base.IStorage_iface);
        } else {
            *result = newTransactedStorage;
        }
    } else {
        *result = &mut (*newStorage).base;
    }

    hr
}

unsafe fn BlockChainStream_GetHeadOfChain(This: *mut BlockChainStream) -> ULONG {
    let mut chainEntry: DirEntry = core::mem::zeroed();
    let hr: HRESULT;

    if (*This).headOfStreamPlaceHolder != core::ptr::null_mut() {
        return *(*This).headOfStreamPlaceHolder;
    }

    if (*This).ownerDirEntry != DIRENTRY_NULL {
        hr = StorageImpl_ReadDirEntry(
            (*This).parentStorage,
            (*This).ownerDirEntry,
            &mut chainEntry,
        );

        if hr >= 0 && chainEntry.startingBlock < BLOCK_FIRST_SPECIAL {
            return chainEntry.startingBlock;
        }
    }

    return BLOCK_END_OF_CHAIN;
}

unsafe fn BlockChainStream_UpdateIndexCache(This: *mut BlockChainStream) -> HRESULT {
    let mut next_sector: ULONG = 0;
    let mut next_offset: ULONG;
    let mut hr: HRESULT;
    let mut last_run: *mut BlockChainRun;

    

    if (*This).indexCacheLen == 0 {
        last_run = core::ptr::null_mut();
        next_offset = 0;
        next_sector = BlockChainStream_GetHeadOfChain(This);
    } else {
        last_run = (*This).indexCache.offset(((*This).indexCacheLen - 1) as isize);
        next_offset = (*last_run).lastOffset + 1;
        hr = StorageImpl_GetNextBlockInChain(
            (*This).parentStorage,
            (*last_run).firstSector + (*last_run).lastOffset - (*last_run).firstOffset,
            &mut next_sector,
        );
        if FAILED(hr) {
            return hr;
        }
    }

    while next_sector != BLOCK_END_OF_CHAIN {
        if last_run.is_null()
            || next_sector != (*last_run).firstSector + next_offset - (*last_run).firstOffset
        {
            if (*This).indexCacheSize == 0 {
                (*This).indexCache = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (core::mem::size_of::<BlockChainRun>() * 16) as SIZE_T,
                ) as *mut BlockChainRun;
                if (*This).indexCache.is_null() {
                    return E_OUTOFMEMORY;
                }
                (*This).indexCacheSize = 16;
            } else if (*This).indexCacheSize == (*This).indexCacheLen {
                let new_cache: *mut BlockChainRun;
                let new_size: ULONG;

                new_size = (*This).indexCacheSize * 2;
                new_cache = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (core::mem::size_of::<BlockChainRun>() * new_size as usize) as SIZE_T,
                ) as *mut BlockChainRun;
                if new_cache.is_null() {
                    return E_OUTOFMEMORY;
                }
                memcpy(
                    new_cache as *mut core::ffi::c_void,
                    (*This).indexCache as *const core::ffi::c_void,
                    core::mem::size_of::<BlockChainRun>() * (*This).indexCacheLen as usize,
                );

                HeapFree(GetProcessHeap(), 0, (*This).indexCache as LPVOID);
                (*This).indexCache = new_cache;
                (*This).indexCacheSize = new_size;
            }

            (*This).indexCacheLen += 1;
            last_run = (*This).indexCache.offset(((*This).indexCacheLen - 1) as isize);
            (*last_run).firstSector = next_sector;
            (*last_run).firstOffset = next_offset;
        }

        (*last_run).lastOffset = next_offset;

        next_offset += 1;
        hr = StorageImpl_GetNextBlockInChain((*This).parentStorage, next_sector, &mut next_sector);
        if FAILED(hr) {
            return hr;
        }
    }

    if (*This).indexCacheLen != 0 {
        (*This).tailIndex = (*last_run).firstSector + (*last_run).lastOffset - (*last_run).firstOffset;
        (*This).numBlocks = (*last_run).lastOffset + 1;
    } else {
        (*This).tailIndex = BLOCK_END_OF_CHAIN;
        (*This).numBlocks = 0;
    }

    S_OK
}

unsafe fn BlockChainStream_Construct(
    parentStorage: *mut StorageImpl,
    headOfStreamPlaceHolder: *mut ULONG,
    dirEntry: DirRef,
) -> *mut BlockChainStream {
    let newStream: *mut BlockChainStream;

    newStream = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<BlockChainStream>()) as SIZE_T) as *mut BlockChainStream;
    if newStream.is_null() {
        return core::ptr::null_mut();
    }

    (*newStream).parentStorage = parentStorage;
    (*newStream).headOfStreamPlaceHolder = headOfStreamPlaceHolder;
    (*newStream).ownerDirEntry = dirEntry;
    (*newStream).indexCache = core::ptr::null_mut();
    (*newStream).indexCacheLen = 0;
    (*newStream).indexCacheSize = 0;
    (*newStream).cachedBlocks[0].index = 0xffffffff;
    (*newStream).cachedBlocks[0].dirty = 0;
    (*newStream).cachedBlocks[1].index = 0xffffffff;
    (*newStream).cachedBlocks[1].dirty = 0;
    (*newStream).blockToEvict = 0;

    if FAILED(BlockChainStream_UpdateIndexCache(newStream)) {
        HeapFree(GetProcessHeap(), 0, (*newStream).indexCache as LPVOID);
        HeapFree(GetProcessHeap(), 0, newStream as LPVOID);
        return core::ptr::null_mut();
    }

    return newStream;
}

unsafe fn BlockChainStream_Shrink(This: *mut BlockChainStream, newSize: ULARGE_INTEGER) -> BOOL {
    let blockIndex: ULONG;
    let mut numBlocks: ULONG;
    let mut i: i32;

    numBlocks = (newSize.QuadPart / (*(*This).parentStorage).bigBlockSize as u64) as ULONG;

    if (newSize.QuadPart % (*(*This).parentStorage).bigBlockSize as u64) != 0 {
        numBlocks += 1;
    }

    if numBlocks != 0 {
        blockIndex = BlockChainStream_GetSectorOfOffset(This, numBlocks - 1);

        StorageImpl_SetNextBlockInChain(
            (*This).parentStorage,
            blockIndex,
            BLOCK_END_OF_CHAIN,
        );

        (*This).tailIndex = blockIndex;
    } else {
        if (*This).headOfStreamPlaceHolder != core::ptr::null_mut() {
            *(*This).headOfStreamPlaceHolder = BLOCK_END_OF_CHAIN;
        } else {
            let mut chainEntry: DirEntry = core::mem::zeroed();
            assert((*This).ownerDirEntry != DIRENTRY_NULL);

            StorageImpl_ReadDirEntry(
                (*This).parentStorage,
                (*This).ownerDirEntry,
                &mut chainEntry,
            );

            chainEntry.startingBlock = BLOCK_END_OF_CHAIN;

            StorageImpl_WriteDirEntry(
                (*This).parentStorage,
                (*This).ownerDirEntry,
                &chainEntry,
            );
        }

        (*This).tailIndex = BLOCK_END_OF_CHAIN;
    }

    (*This).numBlocks = numBlocks;

    while (*This).indexCacheLen != 0 && (*(*This).indexCache.add(((*This).indexCacheLen - 1) as usize)).lastOffset >= numBlocks {
        let last_run: *mut BlockChainRun = (*This).indexCache.add(((*This).indexCacheLen - 1) as usize);
        StorageImpl_FreeBigBlock(
            (*This).parentStorage,
            (*last_run).firstSector + (*last_run).lastOffset - (*last_run).firstOffset,
        );
        if (*last_run).lastOffset == (*last_run).firstOffset {
            (*This).indexCacheLen -= 1;
        } else {
            (*last_run).lastOffset -= 1;
        }
    }

    i = 0;
    while i < 2 {
        if (*This).cachedBlocks[i as usize].index >= numBlocks {
            (*This).cachedBlocks[i as usize].index = 0xffffffff;
            (*This).cachedBlocks[i as usize].dirty = 0;
        }
        i += 1;
    }

    1
}

unsafe fn BlockChainStream_Enlarge(
    This: *mut BlockChainStream,
    newSize: ULARGE_INTEGER,
) -> BOOL {
    let mut blockIndex: ULONG;
    let mut currentBlock: ULONG;
    let mut newNumBlocks: ULONG;
    let mut oldNumBlocks: ULONG;

    blockIndex = BlockChainStream_GetHeadOfChain(This);

    if blockIndex == BLOCK_END_OF_CHAIN {
        blockIndex = StorageImpl_GetNextFreeBigBlock((*This).parentStorage, 1);
        StorageImpl_SetNextBlockInChain(
            (*This).parentStorage,
            blockIndex,
            BLOCK_END_OF_CHAIN,
        );

        if (*This).headOfStreamPlaceHolder != core::ptr::null_mut() {
            *(*This).headOfStreamPlaceHolder = blockIndex;
        } else {
            let mut chainEntry: DirEntry = core::mem::zeroed();
            assert((*This).ownerDirEntry != DIRENTRY_NULL);

            StorageImpl_ReadDirEntry(
                (*This).parentStorage,
                (*This).ownerDirEntry,
                &mut chainEntry,
            );

            chainEntry.startingBlock = blockIndex;

            StorageImpl_WriteDirEntry(
                (*This).parentStorage,
                (*This).ownerDirEntry,
                &chainEntry,
            );
        }

        (*This).tailIndex = blockIndex;
        (*This).numBlocks = 1;
    }

    newNumBlocks = (newSize.QuadPart / (*(*This).parentStorage).bigBlockSize as ULONGLONG) as ULONG;

    if (newSize.QuadPart % (*(*This).parentStorage).bigBlockSize as u64) != 0 {
        newNumBlocks += 1;
    }

    if (*This).tailIndex == BLOCK_END_OF_CHAIN {
        currentBlock = blockIndex;

        while blockIndex != BLOCK_END_OF_CHAIN {
            (*This).numBlocks += 1;
            currentBlock = blockIndex;

            if FAILED(StorageImpl_GetNextBlockInChain(
                (*This).parentStorage,
                currentBlock,
                &mut blockIndex,
            )) {
                return 0;
            }
        }

        (*This).tailIndex = currentBlock;
    }

    currentBlock = (*This).tailIndex;
    oldNumBlocks = (*This).numBlocks;

    if oldNumBlocks < newNumBlocks {
        while oldNumBlocks < newNumBlocks {
            blockIndex = StorageImpl_GetNextFreeBigBlock((*This).parentStorage, newNumBlocks - oldNumBlocks);

            StorageImpl_SetNextBlockInChain(
                (*This).parentStorage,
                currentBlock,
                blockIndex,
            );

            StorageImpl_SetNextBlockInChain(
                (*This).parentStorage,
                blockIndex,
                BLOCK_END_OF_CHAIN,
            );

            currentBlock = blockIndex;
            oldNumBlocks += 1;
        }

        (*This).tailIndex = blockIndex;
        (*This).numBlocks = newNumBlocks;
    }

    if FAILED(BlockChainStream_UpdateIndexCache(This)) {
        return 0;
    }

    1
}

unsafe fn BlockChainStream_GetSize(This: *mut BlockChainStream) -> ULARGE_INTEGER {
    let mut chainEntry: DirEntry = core::mem::zeroed();

    if (*This).headOfStreamPlaceHolder.is_null() {
        StorageImpl_ReadDirEntry(
            (*This).parentStorage,
            (*This).ownerDirEntry,
            &mut chainEntry,
        );

        return chainEntry.size;
    } else {
        let mut result: ULARGE_INTEGER = core::mem::zeroed();
        result.QuadPart = 
            BlockChainStream_GetCount(This) as ULONGLONG *
            (*(*This).parentStorage).bigBlockSize as ULONGLONG;

        return result;
    }
}

unsafe fn BlockChainStream_SetSize(
    This: *mut BlockChainStream,
    newSize: ULARGE_INTEGER,
) -> BOOL {
    let size = BlockChainStream_GetSize(This);

    if newSize.QuadPart == size.QuadPart {
        return 1;
    }

    if newSize.QuadPart < size.QuadPart {
        BlockChainStream_Shrink(This, newSize);
    } else {
        BlockChainStream_Enlarge(This, newSize);
    }

    1
}

unsafe fn BlockChainStream_ReadAt(
    This: *mut BlockChainStream,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *mut core::ffi::c_void,
    bytesRead: *mut ULONG,
) -> HRESULT {
    let mut blockNoInSequence = offset.QuadPart / (*(*This).parentStorage).bigBlockSize as u64;
    let mut offsetInBlock = (offset.QuadPart % (*(*This).parentStorage).bigBlockSize as u64) as ULONG;
    let mut bytesToReadInBuffer: ULONG;
    let mut blockIndex: ULONG;
    let mut bufferWalker: *mut u8;
    let stream_size: ULARGE_INTEGER;
    let mut hr: HRESULT;
    let mut cachedBlock: *mut BlockChainBlock = core::ptr::null_mut();
    let mut size = size;

    blockIndex = BlockChainStream_GetSectorOfOffset(This, blockNoInSequence as ULONG);

    *bytesRead = 0;

    stream_size = BlockChainStream_GetSize(This);
    if stream_size.QuadPart > offset.QuadPart {
        let diff = stream_size.QuadPart - offset.QuadPart;
        if diff < size as u64 {
            size = diff as ULONG;
        }
    } else {
        return 0;
    }

    bufferWalker = buffer as *mut u8;

    while size > 0 {
        let mut ulOffset: ULARGE_INTEGER = core::mem::zeroed();
        let mut bytesReadAt: u32 = 0;

        bytesToReadInBuffer = {
            let remaining_in_block = (*(*This).parentStorage).bigBlockSize - offsetInBlock;
            if remaining_in_block < size {
                remaining_in_block
            } else {
                size
            }
        };

        hr = BlockChainStream_GetBlockAtOffset(
            This,
            blockNoInSequence as ULONG,
            &mut cachedBlock,
            &mut blockIndex,
            if size == bytesToReadInBuffer { 1 } else { 0 },
        );

        if hr < 0 {
            return hr;
        }

        if cachedBlock.is_null() {
            ulOffset.QuadPart = StorageImpl_GetBigBlockOffset((*This).parentStorage, blockIndex) + offsetInBlock as u64;

            StorageImpl_ReadAt(
                (*This).parentStorage,
                ulOffset,
                bufferWalker as *mut core::ffi::c_void,
                bytesToReadInBuffer,
                &mut bytesReadAt,
            );
        } else {
            if (*cachedBlock).read == 0 {
                let mut read: ULONG = 0;
                let read_result = StorageImpl_ReadBigBlock(
                    (*This).parentStorage,
                    (*cachedBlock).sector,
                    (*cachedBlock).data.as_mut_ptr() as *mut core::ffi::c_void,
                    &mut read,
                );
                if read_result < 0 && read == 0 {
                    // C: STG_E_READFAULT (was 0x80030004, STG_E_TOOMANYOPENFILES)
                    return STG_E_READFAULT;
                }

                (*cachedBlock).read = 1;
            }

            core::ptr::copy_nonoverlapping(
                (*cachedBlock).data.as_ptr().add(offsetInBlock as usize),
                bufferWalker,
                bytesToReadInBuffer as usize,
            );
            bytesReadAt = bytesToReadInBuffer;
        }

        blockNoInSequence += 1;
        bufferWalker = bufferWalker.add(bytesReadAt as usize);
        size -= bytesReadAt;
        *bytesRead += bytesReadAt;
        offsetInBlock = 0;

        if bytesToReadInBuffer != bytesReadAt {
            break;
        }
    }

    0
}

pub unsafe fn StorageBaseImpl_RemoveStream(_stg: *mut StorageBaseImpl, strm: *mut StgStreamImpl) {
    TRACE!(b"Stream removed (stg=%p strm=%p)\n\0".as_ptr() as PCHAR);
    list_remove(core::ptr::addr_of_mut!((*strm).StrmListEntry));
}

unsafe fn SmallBlockChainStream_GetHeadOfChain(
    This: *mut SmallBlockChainStream
) -> ULONG {
    let mut chainEntry: DirEntry = core::mem::zeroed();
    let hr: HRESULT;

    if (*This).headOfStreamPlaceHolder != core::ptr::null_mut() {
        return *(*This).headOfStreamPlaceHolder;
    }

    if (*This).ownerDirEntry != 0 {
        hr = StorageImpl_ReadDirEntry(
            (*This).parentStorage,
            (*This).ownerDirEntry,
            &mut chainEntry
        );

        if hr >= 0 && chainEntry.startingBlock < BLOCK_FIRST_SPECIAL {
            return chainEntry.startingBlock;
        }
    }

    return BLOCK_END_OF_CHAIN;
}

unsafe fn SmallBlockChainStream_GetNextBlockInChain(
    This: *mut SmallBlockChainStream,
    blockIndex: ULONG,
    nextBlockInChain: *mut ULONG,
) -> HRESULT {
    let mut offsetOfBlockInDepot: ULARGE_INTEGER = core::mem::zeroed();
    let mut buffer: DWORD = 0;
    let mut bytesRead: ULONG = 0;
    let mut res: HRESULT;

    *nextBlockInChain = BLOCK_END_OF_CHAIN;

    offsetOfBlockInDepot.QuadPart = (blockIndex as ULONGLONG) * (core::mem::size_of::<ULONG>() as ULONGLONG);

    res = BlockChainStream_ReadAt(
        (*(*This).parentStorage).smallBlockDepotChain,
        offsetOfBlockInDepot,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(bytesRead),
    );

    if SUCCEEDED(res) && bytesRead != core::mem::size_of::<DWORD>() as ULONG {
        res = STG_E_READFAULT;
    }

    if SUCCEEDED(res) {
        StorageUtl_ReadDWord(core::ptr::addr_of!(buffer) as *const BYTE, 0, nextBlockInChain);
        return S_OK;
    }

    res
}

unsafe fn SmallBlockChainStream_GetNextFreeBlock(
    This: *mut SmallBlockChainStream,
) -> ULONG {
    let mut offsetOfBlockInDepot: ULARGE_INTEGER = core::mem::zeroed();
    let mut buffer: DWORD = 0;
    let mut bytesRead: ULONG = 0;
    let mut blockIndex: ULONG = (*(*This).parentStorage).firstFreeSmallBlock;
    let mut nextBlockIndex: ULONG = BLOCK_END_OF_CHAIN;
    let mut res: HRESULT;
    let smallBlocksPerBigBlock: ULONG;
    let mut rootEntry: DirEntry = core::mem::zeroed();
    let blocksRequired: ULONG;
    let old_size: ULARGE_INTEGER;
    let mut size_required: ULARGE_INTEGER = core::mem::zeroed();

    offsetOfBlockInDepot.u.HighPart = 0;

    while nextBlockIndex != BLOCK_UNUSED {
        offsetOfBlockInDepot.QuadPart = (blockIndex as ULONGLONG) * (core::mem::size_of::<ULONG>() as ULONGLONG);

        res = BlockChainStream_ReadAt(
            (*(*This).parentStorage).smallBlockDepotChain,
            offsetOfBlockInDepot,
            core::mem::size_of::<DWORD>() as ULONG,
            core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(bytesRead),
        );

        if SUCCEEDED(res) && bytesRead == core::mem::size_of::<DWORD>() as ULONG {
            StorageUtl_ReadDWord(
                core::ptr::addr_of!(buffer) as *const BYTE,
                0,
                core::ptr::addr_of_mut!(nextBlockIndex),
            );

            if nextBlockIndex != BLOCK_UNUSED {
                blockIndex += 1;
            }
        } else {
            let count: ULONG = BlockChainStream_GetCount((*(*This).parentStorage).smallBlockDepotChain);

            let mut smallBlockDepot: [BYTE; MAX_BIG_BLOCK_SIZE as usize] = [0; MAX_BIG_BLOCK_SIZE as usize];
            let mut newSize: ULARGE_INTEGER = core::mem::zeroed();
            let mut offset: ULARGE_INTEGER = core::mem::zeroed();
            let mut bytesWritten: ULONG = 0;

            newSize.QuadPart = ((count + 1) as ULONGLONG) * ((*(*This).parentStorage).bigBlockSize as ULONGLONG);
            BlockChainStream_Enlarge((*(*This).parentStorage).smallBlockDepotChain, newSize);

            core::ptr::write_bytes(
                smallBlockDepot.as_mut_ptr(),
                BLOCK_UNUSED as u8,
                (*(*This).parentStorage).bigBlockSize as usize,
            );
            offset.QuadPart = (count as ULONGLONG) * ((*(*This).parentStorage).bigBlockSize as ULONGLONG);
            BlockChainStream_WriteAt(
                (*(*This).parentStorage).smallBlockDepotChain,
                offset,
                (*(*This).parentStorage).bigBlockSize,
                smallBlockDepot.as_ptr() as *const core::ffi::c_void,
                core::ptr::addr_of_mut!(bytesWritten),
            );

            StorageImpl_SaveFileHeader((*This).parentStorage);
        }
    }

    (*(*This).parentStorage).firstFreeSmallBlock = blockIndex + 1;

    smallBlocksPerBigBlock = (*(*This).parentStorage).bigBlockSize / (*(*This).parentStorage).smallBlockSize;

    blocksRequired = (blockIndex / smallBlocksPerBigBlock) + 1;

    size_required.QuadPart = (blocksRequired as ULONGLONG) * ((*(*This).parentStorage).bigBlockSize as ULONGLONG);

    old_size = BlockChainStream_GetSize((*(*This).parentStorage).smallBlockRootChain);

    if size_required.QuadPart > old_size.QuadPart {
        BlockChainStream_SetSize(
            (*(*This).parentStorage).smallBlockRootChain,
            size_required,
        );

        StorageImpl_ReadDirEntry(
            (*This).parentStorage,
            (*(*This).parentStorage).base.storageDirEntry,
            core::ptr::addr_of_mut!(rootEntry),
        );

        rootEntry.size = size_required;

        StorageImpl_WriteDirEntry(
            (*This).parentStorage,
            (*(*This).parentStorage).base.storageDirEntry,
            core::ptr::addr_of!(rootEntry),
        );
    }

    blockIndex
}

unsafe fn SmallBlockChainStream_ReadAt(
    This: *mut SmallBlockChainStream,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *mut core::ffi::c_void,
    bytesRead: *mut ULONG,
) -> HRESULT {
    let mut rc: HRESULT;
    let mut offsetInBigBlockFile: ULARGE_INTEGER = core::mem::zeroed();
    let mut blockNoInSequence: ULONG = offset.u.LowPart / (*(*This).parentStorage).smallBlockSize;
    let mut offsetInBlock: ULONG = offset.u.LowPart % (*(*This).parentStorage).smallBlockSize;
    let mut bytesToReadInBuffer: ULONG;
    let mut blockIndex: ULONG;
    let _bytesReadFromBigBlockFile: ULONG = 0;
    let mut bufferWalker: *mut BYTE;
    let stream_size: ULARGE_INTEGER;
    let mut size = size;

    

    assert(offset.u.HighPart == 0);

    *bytesRead = 0;

    stream_size = SmallBlockChainStream_GetSize(This);
    if stream_size.QuadPart > offset.QuadPart {
        size = min(stream_size.QuadPart - offset.QuadPart, size as ULONGLONG) as ULONG;
    } else {
        return S_OK;
    }

    blockIndex = SmallBlockChainStream_GetHeadOfChain(This);

    while blockNoInSequence > 0 && blockIndex != BLOCK_END_OF_CHAIN {
        rc = SmallBlockChainStream_GetNextBlockInChain(This, blockIndex, core::ptr::addr_of_mut!(blockIndex));
        if FAILED(rc) {
            return rc;
        }
        blockNoInSequence -= 1;
    }

    bufferWalker = buffer as *mut BYTE;

    while size > 0 && blockIndex != BLOCK_END_OF_CHAIN {
        bytesToReadInBuffer = min((*(*This).parentStorage).smallBlockSize - offsetInBlock, size);

        offsetInBigBlockFile.QuadPart = (blockIndex as ULONGLONG) * ((*(*This).parentStorage).smallBlockSize as ULONGLONG);
        offsetInBigBlockFile.QuadPart += offsetInBlock as ULONGLONG;

        let mut bytesReadFromBigBlockFile_local: ULONG = 0;
        rc = BlockChainStream_ReadAt(
            (*(*This).parentStorage).smallBlockRootChain,
            offsetInBigBlockFile,
            bytesToReadInBuffer,
            bufferWalker as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(bytesReadFromBigBlockFile_local),
        );

        if FAILED(rc) {
            return rc;
        }

        if bytesReadFromBigBlockFile_local == 0 {
            return STG_E_DOCFILECORRUPT;
        }

        rc = SmallBlockChainStream_GetNextBlockInChain(This, blockIndex, core::ptr::addr_of_mut!(blockIndex));
        if FAILED(rc) {
            return STG_E_DOCFILECORRUPT;
        }

        bufferWalker = bufferWalker.add(bytesReadFromBigBlockFile_local as usize);
        size -= bytesReadFromBigBlockFile_local;
        *bytesRead += bytesReadFromBigBlockFile_local;
        offsetInBlock = (offsetInBlock + bytesReadFromBigBlockFile_local) % (*(*This).parentStorage).smallBlockSize;
    }

    S_OK
}

unsafe fn SmallBlockChainStream_WriteAt(
    This: *mut SmallBlockChainStream,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *const core::ffi::c_void,
    bytesWritten: *mut ULONG,
) -> HRESULT {
    

    let mut offsetInBigBlockFile: ULARGE_INTEGER = core::mem::zeroed();
    let mut blockNoInSequence: ULONG = 
        offset.u.LowPart / (*(*This).parentStorage).smallBlockSize;

    let mut offsetInBlock: ULONG = 
        offset.u.LowPart % (*(*This).parentStorage).smallBlockSize;
    let mut bytesToWriteInBuffer: ULONG;
    let mut blockIndex: ULONG;
    let mut bytesWrittenToBigBlockFile: ULONG = 0;
    let mut bufferWalker: *const BYTE;
    let mut res: HRESULT;
    let mut size = size;

    assert((offset.u.HighPart == 0) as i32);

    blockIndex = SmallBlockChainStream_GetHeadOfChain(This);

    while blockNoInSequence > 0 && blockIndex != BLOCK_END_OF_CHAIN {
        if FAILED(SmallBlockChainStream_GetNextBlockInChain(
            This,
            blockIndex,
            &mut blockIndex,
        ))
        {
            return STG_E_DOCFILECORRUPT;
        }
        blockNoInSequence -= 1;
    }

    *bytesWritten = 0;
    bufferWalker = buffer as *const BYTE;
    while size > 0 && blockIndex != BLOCK_END_OF_CHAIN {
        bytesToWriteInBuffer = min(
            (*(*This).parentStorage).smallBlockSize - offsetInBlock,
            size,
        );

        offsetInBigBlockFile.QuadPart =
            (blockIndex as ULONGLONG) * ((*(*This).parentStorage).smallBlockSize as ULONGLONG);

        offsetInBigBlockFile.QuadPart += offsetInBlock as ULONGLONG;

        res = BlockChainStream_WriteAt(
            (*(*This).parentStorage).smallBlockRootChain,
            offsetInBigBlockFile,
            bytesToWriteInBuffer,
            bufferWalker as *const core::ffi::c_void,
            &mut bytesWrittenToBigBlockFile,
        );
        if FAILED(res) {
            return res;
        }

        res = SmallBlockChainStream_GetNextBlockInChain(This, blockIndex, &mut blockIndex);
        if FAILED(res) {
            return res;
        }
        bufferWalker = bufferWalker.add(bytesWrittenToBigBlockFile as usize);
        size -= bytesWrittenToBigBlockFile;
        *bytesWritten += bytesWrittenToBigBlockFile;
        offsetInBlock = (offsetInBlock + bytesWrittenToBigBlockFile)
            % (*(*This).parentStorage).smallBlockSize;
    }

    if size == 0 {
        S_OK
    } else {
        STG_E_WRITEFAULT
    }
}

unsafe fn SmallBlockChainStream_Shrink(
    This: *mut SmallBlockChainStream,
    newSize: ULARGE_INTEGER,
) -> BOOL {
    let mut blockIndex: ULONG;
    let mut extraBlock: ULONG = 0;
    let mut numBlocks: ULONG;
    let mut count: ULONG = 0;

    numBlocks = newSize.u.LowPart / (*(*This).parentStorage).smallBlockSize;

    if (newSize.u.LowPart % (*(*This).parentStorage).smallBlockSize) != 0 {
        numBlocks += 1;
    }

    blockIndex = SmallBlockChainStream_GetHeadOfChain(This);

    while count < numBlocks {
        if FAILED(SmallBlockChainStream_GetNextBlockInChain(
            This,
            blockIndex,
            core::ptr::addr_of_mut!(blockIndex),
        )) {
            return 0;
        }
        count += 1;
    }

    if count == 0 {
        let mut chainEntry: DirEntry = core::mem::zeroed();

        StorageImpl_ReadDirEntry(
            (*This).parentStorage,
            (*This).ownerDirEntry,
            core::ptr::addr_of_mut!(chainEntry),
        );

        chainEntry.startingBlock = BLOCK_END_OF_CHAIN;

        StorageImpl_WriteDirEntry(
            (*This).parentStorage,
            (*This).ownerDirEntry,
            core::ptr::addr_of!(chainEntry),
        );

        extraBlock = blockIndex;
    } else {
        if FAILED(SmallBlockChainStream_GetNextBlockInChain(
            This,
            blockIndex,
            core::ptr::addr_of_mut!(extraBlock),
        )) {
            return 0;
        }

        SmallBlockChainStream_SetNextBlockInChain(This, blockIndex, BLOCK_END_OF_CHAIN);
    }

    while extraBlock != BLOCK_END_OF_CHAIN {
        if FAILED(SmallBlockChainStream_GetNextBlockInChain(
            This,
            extraBlock,
            core::ptr::addr_of_mut!(blockIndex),
        )) {
            return 0;
        }
        SmallBlockChainStream_FreeBlock(This, extraBlock);
        (*(*This).parentStorage).firstFreeSmallBlock = core::cmp::min(
            (*(*This).parentStorage).firstFreeSmallBlock,
            extraBlock,
        );
        extraBlock = blockIndex;
    }

    1
}

unsafe fn SmallBlockChainStream_Enlarge(
    This: *mut SmallBlockChainStream,
    newSize: ULARGE_INTEGER,
) -> BOOL {
    let mut blockIndex: ULONG;
    let mut currentBlock: ULONG;
    let mut newNumBlocks: ULONG;
    let mut oldNumBlocks: ULONG = 0;

    blockIndex = SmallBlockChainStream_GetHeadOfChain(This);

    if blockIndex == BLOCK_END_OF_CHAIN {
        blockIndex = SmallBlockChainStream_GetNextFreeBlock(This);
        SmallBlockChainStream_SetNextBlockInChain(
            This,
            blockIndex,
            BLOCK_END_OF_CHAIN,
        );

        if (*This).headOfStreamPlaceHolder != core::ptr::null_mut() {
            *(*This).headOfStreamPlaceHolder = blockIndex;
        } else {
            let mut chainEntry: DirEntry = core::mem::zeroed();

            StorageImpl_ReadDirEntry(
                (*This).parentStorage,
                (*This).ownerDirEntry,
                &mut chainEntry,
            );

            chainEntry.startingBlock = blockIndex;

            StorageImpl_WriteDirEntry(
                (*This).parentStorage,
                (*This).ownerDirEntry,
                &chainEntry,
            );
        }
    }

    currentBlock = blockIndex;

    newNumBlocks = newSize.u.LowPart / (*(*This).parentStorage).smallBlockSize;

    if (newSize.u.LowPart % (*(*This).parentStorage).smallBlockSize) != 0 {
        newNumBlocks += 1;
    }

    while blockIndex != BLOCK_END_OF_CHAIN {
        oldNumBlocks += 1;
        currentBlock = blockIndex;
        if FAILED(SmallBlockChainStream_GetNextBlockInChain(
            This,
            currentBlock,
            &mut blockIndex,
        )) {
            return 0;
        }
    }

    while oldNumBlocks < newNumBlocks {
        blockIndex = SmallBlockChainStream_GetNextFreeBlock(This);
        SmallBlockChainStream_SetNextBlockInChain(This, currentBlock, blockIndex);

        SmallBlockChainStream_SetNextBlockInChain(
            This,
            blockIndex,
            BLOCK_END_OF_CHAIN,
        );

        currentBlock = blockIndex;
        oldNumBlocks += 1;
    }

    1
}

unsafe fn SmallBlockChainStream_SetSize(
    This: *mut SmallBlockChainStream,
    newSize: ULARGE_INTEGER,
) -> BOOL {
    let size = SmallBlockChainStream_GetSize(This);

    if newSize.u.LowPart == size.u.LowPart {
        return 1;
    }

    if newSize.u.LowPart < size.u.LowPart {
        SmallBlockChainStream_Shrink(This, newSize);
    } else {
        SmallBlockChainStream_Enlarge(This, newSize);
    }

    1
}

unsafe fn SmallBlockChainStream_GetCount(This: *mut SmallBlockChainStream) -> ULONG {
    let mut blockIndex: ULONG;
    let mut count: ULONG = 0;

    blockIndex = SmallBlockChainStream_GetHeadOfChain(This);

    while blockIndex != BLOCK_END_OF_CHAIN {
        count += 1;

        if FAILED(SmallBlockChainStream_GetNextBlockInChain(
            This,
            blockIndex,
            core::ptr::addr_of_mut!(blockIndex),
        )) {
            return 0;
        }
    }

    return count;
}

unsafe fn SmallBlockChainStream_GetSize(This: *mut SmallBlockChainStream) -> ULARGE_INTEGER {
    let mut chainEntry: DirEntry = core::mem::zeroed();

    if (*This).headOfStreamPlaceHolder != core::ptr::null_mut() {
        let mut result: ULARGE_INTEGER = core::mem::zeroed();
        result.u.HighPart = 0;

        result.u.LowPart = SmallBlockChainStream_GetCount(This)
            * (*(*This).parentStorage).smallBlockSize;

        return result;
    }

    StorageImpl_ReadDirEntry(
        (*This).parentStorage,
        (*This).ownerDirEntry,
        &mut chainEntry,
    );

    chainEntry.size
}

unsafe fn StorageBaseImpl_CopyStream(
    dst: *mut StorageBaseImpl,
    dst_entry: DirRef,
    src: *mut StorageBaseImpl,
    src_entry: DirRef,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut data: [u8; 4096] = [0; 4096];
    let mut srcdata: DirEntry = core::mem::zeroed();
    let mut bytes_copied: ULARGE_INTEGER = core::mem::zeroed();
    let mut bytestocopy: ULONG;
    let mut bytesread: ULONG = 0;
    let mut byteswritten: ULONG = 0;

    hr = StorageBaseImpl_ReadDirEntry(src, src_entry, &mut srcdata);

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_StreamSetSize(dst, dst_entry, srcdata.size);

        bytes_copied.QuadPart = 0;
        while bytes_copied.QuadPart < srcdata.size.QuadPart && SUCCEEDED(hr) {
            bytestocopy = min(4096, (srcdata.size.QuadPart - bytes_copied.QuadPart) as ULONG);

            hr = StorageBaseImpl_StreamReadAt(
                src,
                src_entry,
                bytes_copied,
                bytestocopy,
                data.as_mut_ptr() as *mut core::ffi::c_void,
                &mut bytesread,
            );
            if SUCCEEDED(hr) && bytesread != bytestocopy {
                hr = STG_E_READFAULT;
            }

            if SUCCEEDED(hr) {
                hr = StorageBaseImpl_StreamWriteAt(
                    dst,
                    dst_entry,
                    bytes_copied,
                    bytestocopy,
                    data.as_ptr() as *const core::ffi::c_void,
                    &mut byteswritten,
                );
            }
            if SUCCEEDED(hr) {
                if byteswritten != bytestocopy {
                    hr = STG_E_WRITEFAULT;
                }
                bytes_copied.QuadPart += byteswritten as ULONGLONG;
            }
        }
    }

    hr
}

unsafe fn create_storagefile(
    pwcsName: LPCOLESTR,
    grfMode: DWORD,
    _grfAttrs: DWORD,
    pStgOptions: *mut STGOPTIONS,
    riid: REFIID,
    ppstgOpen: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut newStorage: *mut StorageBaseImpl = core::ptr::null_mut();
    let hFile: HANDLE;
    let mut hr: HRESULT;
    let shareMode: DWORD;
    let accessMode: DWORD;
    let creationMode: DWORD;
    let fileAttributes: DWORD;
    let mut tempFileName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if ppstgOpen == core::ptr::null_mut() {
        return STG_E_INVALIDPOINTER;
    }

    if (*pStgOptions).ulSectorSize != MIN_BIG_BLOCK_SIZE && (*pStgOptions).ulSectorSize != MAX_BIG_BLOCK_SIZE {
        return STG_E_INVALIDPARAMETER;
    }

    let mut grfMode = grfMode;
    if STGM_SHARE_MODE(grfMode) == 0 {
        grfMode |= STGM_SHARE_DENY_NONE;
    }

    if FAILED(validateSTGM(grfMode)) {
        hr = STG_E_INVALIDFLAG;
        TRACE!(b"<-- %p  r = %08x\n\0".as_ptr() as PCHAR);
        return hr;
    }

    match STGM_ACCESS_MODE(grfMode) {
        STGM_WRITE | STGM_READWRITE => {},
        _ => {
            hr = STG_E_INVALIDFLAG;
            TRACE!(b"<-- %p  r = %08x\n\0".as_ptr() as PCHAR);
            return hr;
        }
    }

    if (grfMode & STGM_TRANSACTED) == 0 && STGM_SHARE_MODE(grfMode) != STGM_SHARE_EXCLUSIVE {
        hr = STG_E_INVALIDFLAG;
        TRACE!(b"<-- %p  r = %08x\n\0".as_ptr() as PCHAR);
        return hr;
    }

    let pwcsName = if pwcsName == core::ptr::null() {
        let mut tempPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        static prefix: [WCHAR; 4] = ['S' as u16, 'T' as u16, 'O' as u16, 0];

        core::ptr::write_bytes(tempPath.as_mut_ptr(), 0, MAX_PATH as usize);
        core::ptr::write_bytes(tempFileName.as_mut_ptr(), 0, MAX_PATH as usize);

        if GetTempPathW(MAX_PATH, tempPath.as_mut_ptr()) == 0 {
            tempPath[0] = '.' as u16;
        }

        if GetTempFileNameW(tempPath.as_ptr(), prefix.as_ptr(), 0, tempFileName.as_mut_ptr()) != 0 {
            tempFileName.as_ptr()
        } else {
            hr = STG_E_INSUFFICIENTMEMORY;
            TRACE!(b"<-- %p  r = %08x\n\0".as_ptr() as PCHAR);
            return hr;
        }
    } else {
        pwcsName
    };

    creationMode = if pwcsName == tempFileName.as_ptr() {
        TRUNCATE_EXISTING
    } else {
        GetCreationModeFromSTGM(grfMode)
    };

    shareMode = GetShareModeFromSTGM(grfMode);
    accessMode = GetAccessModeFromSTGM(grfMode);

    if (grfMode & STGM_DELETEONRELEASE) != 0 {
        fileAttributes = FILE_FLAG_RANDOM_ACCESS | FILE_FLAG_DELETE_ON_CLOSE;
    } else {
        fileAttributes = FILE_ATTRIBUTE_NORMAL | FILE_FLAG_RANDOM_ACCESS;
    }

    *ppstgOpen = core::ptr::null_mut();

    hFile = CreateFileW(
        pwcsName,
        accessMode,
        shareMode,
        core::ptr::null_mut(),
        creationMode,
        fileAttributes,
        core::ptr::null_mut(),
    );

    if hFile == INVALID_HANDLE_VALUE {
        hr = if GetLastError() == ERROR_FILE_EXISTS {
            STG_E_FILEALREADYEXISTS
        } else {
            E_FAIL
        };
        TRACE!(b"<-- %p  r = %08x\n\0".as_ptr() as PCHAR);
        return hr;
    }

    hr = Storage_Construct(
        hFile,
        pwcsName,
        core::ptr::null_mut(),
        grfMode,
        1,
        1,
        (*pStgOptions).ulSectorSize,
        &mut newStorage,
    );

    if FAILED(hr) {
        TRACE!(b"<-- %p  r = %08x\n\0".as_ptr() as PCHAR);
        return hr;
    }

    hr = IStorage_QueryInterface(
        core::ptr::addr_of_mut!((*newStorage).IStorage_iface),
        riid,
        ppstgOpen,
    );
    IStorage_Release(core::ptr::addr_of_mut!((*newStorage).IStorage_iface));

    TRACE!(b"<-- %p  r = %08x\n\0".as_ptr() as PCHAR);

    hr
}

/// ole32.spec:293 `@ stdcall StgCreateDocfile(wstr long long ptr)` — a DLL EXPORT. The
/// translation dropped both `#[no_mangle]` and the ABI, leaving a module-private Rust-ABI
/// fn, so `_StgCreateDocfile@16` was simply absent from the link.
#[no_mangle]
pub unsafe extern "system" fn StgCreateDocfile(
    pwcsName: LPCOLESTR,
    grfMode: DWORD,
    reserved: DWORD,
    ppstgOpen: *mut *mut IStorage,
) -> HRESULT {
    let mut stgoptions = STGOPTIONS {
        usVersion: 1,
        reserved: 0,
        ulSectorSize: 512,
        pwcsTemplateFile: core::ptr::null(),
    };

    

    TRACE!(b"(%s, %x, %d, %p)\n\0".as_ptr() as LPCSTR);

    if ppstgOpen.is_null() {
        return STG_E_INVALIDPOINTER;
    }
    if reserved != 0 {
        return STG_E_INVALIDPARAMETER;
    }

    create_storagefile(
        pwcsName,
        grfMode,
        0,
        &mut stgoptions,
        &IID_IStorage,
        ppstgOpen as *mut *mut core::ffi::c_void,
    )
}

#[no_mangle]
pub unsafe extern "system" fn StgCreateStorageEx(
    pwcsName: *const WCHAR,
    grfMode: DWORD,
    stgfmt: DWORD,
    grfAttrs: DWORD,
    pStgOptions: *mut STGOPTIONS,
    _reserved: *mut core::ffi::c_void,
    riid: REFIID,
    ppObjectOpen: *mut *mut core::ffi::c_void,
) -> HRESULT {
    
    
    TRACE!(b"(%s, %x, %x, %x, %p, %p, %p, %p)\n\0".as_ptr() as LPCSTR);

    if stgfmt != STGFMT_FILE && grfAttrs != 0 {
        ERR(b"grfAttrs must be 0 if stgfmt != STGFMT_FILE\n\0".as_ptr() as LPCSTR);
        return STG_E_INVALIDPARAMETER;
    }

    if stgfmt == STGFMT_FILE && grfAttrs != 0 && grfAttrs != FILE_FLAG_NO_BUFFERING {
        ERR(b"grfAttrs must be 0 or FILE_FLAG_NO_BUFFERING if stgfmt == STGFMT_FILE\n\0".as_ptr() as LPCSTR);
        return STG_E_INVALIDPARAMETER;
    }

    if stgfmt == STGFMT_FILE {
        ERR(b"Cannot use STGFMT_FILE - this is NTFS only\n\0".as_ptr() as LPCSTR);
        return STG_E_INVALIDPARAMETER;
    }

    if stgfmt == STGFMT_STORAGE || stgfmt == STGFMT_DOCFILE {
        let mut defaultOptions = STGOPTIONS {
            usVersion: 1,
            reserved: 0,
            ulSectorSize: 512,
            pwcsTemplateFile: core::ptr::null(),
        };

        let pStgOptions = if pStgOptions.is_null() {
            core::ptr::addr_of_mut!(defaultOptions)
        } else {
            pStgOptions
        };
        
        return create_storagefile(pwcsName, grfMode, grfAttrs, pStgOptions, riid, ppObjectOpen);
    }

    ERR(b"Invalid stgfmt argument\n\0".as_ptr() as LPCSTR);
    STG_E_INVALIDPARAMETER
}

#[no_mangle]
pub unsafe extern "system" fn StgOpenStorageEx(
    pwcsName: *const u16,
    grfMode: u32,
    stgfmt: u32,
    grfAttrs: u32,
    _pStgOptions: *mut STGOPTIONS,
    _reserved: *mut core::ffi::c_void,
    _riid: *const IID,
    ppObjectOpen: *mut *mut core::ffi::c_void,
) -> HRESULT {
    

    TRACE!(b"(%s, %x, %x, %x, %p, %p, %p, %p)\n\0".as_ptr() as *const c_char);

    if stgfmt != STGFMT_DOCFILE && grfAttrs != 0 {
        ERR(b"grfAttrs must be 0 if stgfmt != STGFMT_DOCFILE\n\0".as_ptr() as *const c_char);
        return STG_E_INVALIDPARAMETER;
    }

    match stgfmt {
        STGFMT_FILE => {
            ERR(b"Cannot use STGFMT_FILE - this is NTFS only\n\0".as_ptr() as *const c_char);
            return STG_E_INVALIDPARAMETER;
        }

        STGFMT_STORAGE => {}

        STGFMT_DOCFILE => {
            if grfAttrs != 0 && grfAttrs != FILE_FLAG_NO_BUFFERING {
                ERR(b"grfAttrs must be 0 or FILE_FLAG_NO_BUFFERING if stgfmt == STGFMT_DOCFILE\n\0".as_ptr() as *const c_char);
                return STG_E_INVALIDPARAMETER;
            }
            FIXME(b"Stub: calling StgOpenStorage, but ignoring pStgOptions and grfAttrs\n\0".as_ptr() as *const c_char);
        }

        STGFMT_ANY => {
            WARN!(b"STGFMT_ANY assuming storage\n\0".as_ptr() as *const c_char);
        }

        _ => {
            return STG_E_INVALIDPARAMETER;
        }
    }

    StgOpenStorage(
        pwcsName,
        core::ptr::null_mut(),
        grfMode,
        core::ptr::null_mut(),
        0,
        ppObjectOpen as *mut *mut IStorage,
    )
}

#[no_mangle]
pub unsafe extern "system" fn StgOpenStorage(
    pwcsName: *const OLECHAR,
    pstgPriority: *mut IStorage,
    grfMode: DWORD,
    _snbExclude: SNB,
    reserved: DWORD,
    ppstgOpen: *mut *mut IStorage,
) -> HRESULT {
    let mut newStorage: *mut StorageBaseImpl = core::ptr::null_mut();
    let mut hr: HRESULT;
    let hFile: HANDLE;
    let shareMode: DWORD;
    let accessMode: DWORD;
    let mut temp_name: LPWSTR = core::ptr::null_mut();

    TRACE!(b"(%s, %p, %x, %p, %d, %p)\n\0".as_ptr() as PCHAR);

    let mut pwcsName = pwcsName;

    if !pstgPriority.is_null() {
        hr = StorageBaseImpl_GetFilename(pstgPriority as *mut StorageBaseImpl, &mut temp_name);
        if FAILED(hr) {
            CoTaskMemFree(temp_name as LPVOID);
            if !pstgPriority.is_null() {
                IStorage_Release(pstgPriority);
            }
            TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
            return hr;
        }
        pwcsName = temp_name;
        TRACE!(b"using filename %s\n\0".as_ptr() as PCHAR);
    }

    if pwcsName.is_null() {
        hr = STG_E_INVALIDNAME;
        CoTaskMemFree(temp_name as LPVOID);
        if !pstgPriority.is_null() {
            IStorage_Release(pstgPriority);
        }
        TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
        return hr;
    }

    if ppstgOpen.is_null() {
        hr = STG_E_INVALIDPOINTER;
        CoTaskMemFree(temp_name as LPVOID);
        if !pstgPriority.is_null() {
            IStorage_Release(pstgPriority);
        }
        TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
        return hr;
    }

    if reserved != 0 {
        hr = STG_E_INVALIDPARAMETER;
        CoTaskMemFree(temp_name as LPVOID);
        if !pstgPriority.is_null() {
            IStorage_Release(pstgPriority);
        }
        TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
        return hr;
    }

    let mut grfMode = grfMode;

    if (grfMode & STGM_PRIORITY) != 0 {
        if (grfMode & (STGM_TRANSACTED | STGM_SIMPLE | STGM_NOSCRATCH | STGM_NOSNAPSHOT)) != 0 {
            return STG_E_INVALIDFLAG;
        }
        if (grfMode & STGM_DELETEONRELEASE) != 0 {
            return STG_E_INVALIDFUNCTION;
        }
        if STGM_ACCESS_MODE(grfMode) != STGM_READ {
            return STG_E_INVALIDFLAG;
        }
        grfMode &= !0xf0;
        grfMode |= STGM_SHARE_DENY_NONE;
    }

    if (grfMode & STGM_DIRECT_SWMR) != 0 {
        if (STGM_SHARE_MODE(grfMode) != STGM_SHARE_DENY_WRITE)
            && (STGM_SHARE_MODE(grfMode) != STGM_SHARE_DENY_NONE)
        {
            hr = STG_E_INVALIDFLAG;
            CoTaskMemFree(temp_name as LPVOID);
            if !pstgPriority.is_null() {
                IStorage_Release(pstgPriority);
            }
            TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
            return hr;
        }
    } else if (grfMode & (STGM_TRANSACTED | STGM_PRIORITY)) == 0 {
        match STGM_SHARE_MODE(grfMode) {
            STGM_SHARE_EXCLUSIVE | STGM_SHARE_DENY_WRITE => {}
            _ => {
                hr = STG_E_INVALIDFLAG;
                CoTaskMemFree(temp_name as LPVOID);
                if !pstgPriority.is_null() {
                    IStorage_Release(pstgPriority);
                }
                TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
                return hr;
            }
        }
    }

    if FAILED(validateSTGM(grfMode)) || (grfMode & STGM_CREATE) != 0 {
        hr = STG_E_INVALIDFLAG;
        CoTaskMemFree(temp_name as LPVOID);
        if !pstgPriority.is_null() {
            IStorage_Release(pstgPriority);
        }
        TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
        return hr;
    }

    if STGM_SHARE_MODE(grfMode) == STGM_SHARE_DENY_WRITE
        && STGM_ACCESS_MODE(grfMode) == STGM_READWRITE
        && (grfMode & STGM_TRANSACTED) == 0
        && (grfMode & STGM_DIRECT_SWMR) == 0
    {
        hr = STG_E_INVALIDFLAG;
        CoTaskMemFree(temp_name as LPVOID);
        if !pstgPriority.is_null() {
            IStorage_Release(pstgPriority);
        }
        TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
        return hr;
    }

    shareMode = GetShareModeFromSTGM(grfMode);
    accessMode = GetAccessModeFromSTGM(grfMode);

    *ppstgOpen = core::ptr::null_mut();

    hFile = CreateFileW(
        pwcsName,
        accessMode,
        shareMode,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_RANDOM_ACCESS,
        core::ptr::null_mut(),
    );

    if hFile == INVALID_HANDLE_VALUE {
        let last_error = GetLastError();


        match last_error {
            ERROR_FILE_NOT_FOUND => {
                hr = STG_E_FILENOTFOUND;
            }
            ERROR_PATH_NOT_FOUND => {
                hr = STG_E_PATHNOTFOUND;
            }
            ERROR_ACCESS_DENIED | ERROR_WRITE_PROTECT => {
                hr = STG_E_ACCESSDENIED;
            }
            ERROR_SHARING_VIOLATION => {
                hr = STG_E_SHAREVIOLATION;
            }
            _ => {
                hr = E_FAIL;
            }
        }

        CoTaskMemFree(temp_name as LPVOID);
        if !pstgPriority.is_null() {
            IStorage_Release(pstgPriority);
        }
        TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
        return hr;
    }

    if GetFileSize(hFile, core::ptr::null_mut()) < HEADER_SIZE {
        CloseHandle(hFile);
        hr = STG_E_FILEALREADYEXISTS;
        CoTaskMemFree(temp_name as LPVOID);
        if !pstgPriority.is_null() {
            IStorage_Release(pstgPriority);
        }
        TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
        return hr;
    }

    hr = Storage_Construct(
        hFile,
        pwcsName,
        core::ptr::null_mut(),
        grfMode,
        1,
        0,
        512,
        &mut newStorage,
    );

    if FAILED(hr) {
        if hr == STG_E_INVALIDHEADER {
            hr = STG_E_FILEALREADYEXISTS;
        }
        CoTaskMemFree(temp_name as LPVOID);
        if !pstgPriority.is_null() {
            IStorage_Release(pstgPriority);
        }
        TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
        return hr;
    }

    *ppstgOpen = core::ptr::addr_of_mut!((*newStorage).IStorage_iface);

    CoTaskMemFree(temp_name as LPVOID);
    if !pstgPriority.is_null() {
        IStorage_Release(pstgPriority);
    }
    TRACE!(b"<-- %08x, IStorage %p\n\0".as_ptr() as PCHAR);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn StgCreateDocfileOnILockBytes(
    plkbyt: *mut ILockBytes,
    grfMode: DWORD,
    _reserved: DWORD,
    ppstgOpen: *mut *mut IStorage,
) -> HRESULT {
    let mut newStorage: *mut StorageBaseImpl = core::ptr::null_mut();
    let hr: HRESULT;

    if ppstgOpen.is_null() || plkbyt.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    // C passes 0 ("no file"): StorageImpl_Construct takes the file branch for any
    // non-NULL handle, so INVALID_HANDLE_VALUE here sent every ILockBytes-backed
    // storage to FileLockBytesImpl_Construct, which rejects it -- the construct
    // failed, and its cleanup then flushed through the NULL lockBytes.
    hr = Storage_Construct(
        core::ptr::null_mut(),
        core::ptr::null(),
        plkbyt,
        grfMode,
        0,
        1,
        512,
        &mut newStorage,
    );

    if FAILED(hr) {
        return hr;
    }

    *ppstgOpen = &mut (*newStorage).IStorage_iface;

    hr
}

#[no_mangle]
pub unsafe extern "system" fn StgOpenStorageOnILockBytes(
    plkbyt: *mut ILockBytes,
    _pstgPriority: *mut IStorage,
    grfMode: DWORD,
    _snbExclude: SNB,
    _reserved: DWORD,
    ppstgOpen: *mut *mut IStorage,
) -> HRESULT {
    let mut newStorage: *mut StorageBaseImpl = core::ptr::null_mut();
    let hr: HRESULT;

    if plkbyt.is_null() || ppstgOpen.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    if FAILED(validateSTGM(grfMode)) {
        return STG_E_INVALIDFLAG;
    }

    *ppstgOpen = core::ptr::null_mut();

    hr = Storage_Construct(
        0 as HANDLE,
        core::ptr::null(),
        plkbyt,
        grfMode,
        0,
        0,
        512,
        &mut newStorage,
    );

    if FAILED(hr) {
        return hr;
    }

    *ppstgOpen = core::ptr::addr_of_mut!((*newStorage).IStorage_iface);

    hr
}

#[no_mangle]
pub unsafe extern "system" fn StgSetTimes(
    str: *const OLECHAR,
    pctime: *const FILETIME,
    patime: *const FILETIME,
    pmtime: *const FILETIME,
) -> HRESULT {
    let mut stg: *mut IStorage = core::ptr::null_mut();
    let mut r: HRESULT;

    TRACE!(b"%s %p %p %p\n\0".as_ptr() as PCHAR);

    r = StgOpenStorage(
        str,
        core::ptr::null_mut(),
        STGM_READWRITE | STGM_SHARE_DENY_WRITE,
        core::ptr::null_mut(),
        0,
        &mut stg,
    );
    if SUCCEEDED(r) {
        r = IStorage_SetElementTimes(stg, core::ptr::null(), pctime, patime, pmtime);
        IStorage_Release(stg);
    }

    r
}

unsafe fn OLECONVERT_GetOLE20FromOLE10(pDestStorage: *mut IStorage, pBuffer: *const BYTE, nBufferLength: DWORD) {
    let hRes: HRESULT;
    let hFile: HANDLE;
    let mut pTempStorage: *mut IStorage = core::ptr::null_mut();
    let mut dwNumOfBytesWritten: DWORD = 0;
    let mut wstrTempDir: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wstrTempFile: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let wstrPrefix: [WCHAR; 4] = [b's' as u16, b'i' as u16, b's' as u16, 0];

    

    GetTempPathW(MAX_PATH, wstrTempDir.as_mut_ptr());
    GetTempFileNameW(wstrTempDir.as_ptr(), wstrPrefix.as_ptr(), 0, wstrTempFile.as_mut_ptr());
    hFile = CreateFileW(wstrTempFile.as_ptr(), GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());

    if hFile != INVALID_HANDLE_VALUE {
        WriteFile(hFile, pBuffer as LPVOID, nBufferLength, &mut dwNumOfBytesWritten, core::ptr::null_mut());
        CloseHandle(hFile);

        hRes = StgOpenStorage(wstrTempFile.as_ptr(), core::ptr::null_mut(), STGM_READ, core::ptr::null_mut(), 0, &mut pTempStorage);
        if hRes == S_OK {
            IStorage_CopyTo(pTempStorage, 0, core::ptr::null(), core::ptr::null_mut(), pDestStorage);
            IStorage_Release(pTempStorage);
        }
        DeleteFileW(wstrTempFile.as_ptr());
    }
}

unsafe fn OLECONVERT_WriteOLE20ToBuffer(pStorage: *mut IStorage, pData: *mut *mut BYTE) -> DWORD {
    let hFile: HANDLE;
    let hRes: HRESULT;
    let mut nDataLength: DWORD = 0;
    let mut pTempStorage: *mut IStorage = core::ptr::null_mut();
    let mut wstrTempDir: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wstrTempFile: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    static wstrPrefix: [WCHAR; 4] = [b's' as u16, b'i' as u16, b's' as u16, 0];

    *pData = core::ptr::null_mut();

    GetTempPathW(MAX_PATH, wstrTempDir.as_mut_ptr());
    GetTempFileNameW(wstrTempDir.as_ptr(), wstrPrefix.as_ptr(), 0, wstrTempFile.as_mut_ptr());
    hRes = StgCreateDocfile(
        wstrTempFile.as_ptr(),
        STGM_CREATE | STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
        0,
        &mut pTempStorage
    );

    if hRes == S_OK {
        IStorage_CopyTo(pStorage, 0, core::ptr::null_mut(), core::ptr::null_mut(), pTempStorage);
        IStorage_Release(pTempStorage);

        hFile = CreateFileW(
            wstrTempFile.as_ptr(),
            GENERIC_READ,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut()
        );
        if hFile != INVALID_HANDLE_VALUE {
            nDataLength = GetFileSize(hFile, core::ptr::null_mut());
            *pData = HeapAlloc(GetProcessHeap(), 0, (nDataLength as usize) as SIZE_T) as *mut BYTE;
            ReadFile(hFile, *pData as LPVOID, nDataLength, &mut nDataLength, core::ptr::null_mut());
            CloseHandle(hFile);
        }
        DeleteFileW(wstrTempFile.as_ptr());
    }
    nDataLength
}

unsafe fn StorageBaseImpl_DeleteStorageTree(
    This: *mut StorageBaseImpl,
    entry: DirRef,
    include_siblings: BOOL,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut data: DirEntry = core::mem::zeroed();
    let mut zero: ULARGE_INTEGER = core::mem::zeroed();

    if entry == DIRENTRY_NULL {
        return S_OK;
    }

    zero.QuadPart = 0;

    hr = StorageBaseImpl_ReadDirEntry(This, entry, &mut data);

    if SUCCEEDED(hr) && include_siblings != 0 {
        hr = StorageBaseImpl_DeleteStorageTree(This, data.leftChild, 1);
    }

    if SUCCEEDED(hr) && include_siblings != 0 {
        hr = StorageBaseImpl_DeleteStorageTree(This, data.rightChild, 1);
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_DeleteStorageTree(This, data.dirRootEntry, 1);
    }

    if SUCCEEDED(hr) && data.stgType as DWORD == STGTY_STREAM {
        hr = StorageBaseImpl_StreamSetSize(This, entry, zero);
    }

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_DestroyDirEntry(This, entry);
    }

    hr
}

unsafe fn OLECONVERT_GetOle20PresData(pStorage: *mut IStorage, pOleStreamData: *mut OLECONVERT_OLESTREAM_DATA) {
    

    let hRes: HRESULT;
    let mut pStream: *mut IStream = core::ptr::null_mut();
    let mut olePress: OLECONVERT_ISTORAGE_OLEPRES = core::mem::zeroed();
    static wstrStreamName: [u16; 12] = [2, b'O' as u16, b'l' as u16, b'e' as u16, b'P' as u16, b'r' as u16, b'e' as u16, b's' as u16, b'0' as u16, b'0' as u16, b'0' as u16, 0];

    (*pOleStreamData.offset(0)).dwOleID = OLESTREAM_ID;
    (*pOleStreamData.offset(0)).dwTypeID = 2;
    (*pOleStreamData.offset(0)).dwMetaFileWidth = 0;
    (*pOleStreamData.offset(0)).dwMetaFileHeight = 0;
    (*pOleStreamData.offset(0)).dwDataLength = OLECONVERT_WriteOLE20ToBuffer(pStorage, core::ptr::addr_of_mut!((*pOleStreamData.offset(0)).pData));
    (*pOleStreamData.offset(1)).dwOleID = OLESTREAM_ID;
    (*pOleStreamData.offset(1)).dwTypeID = 0;
    (*pOleStreamData.offset(1)).dwOleTypeNameLength = 0;
    (*pOleStreamData.offset(1)).strOleTypeName[0] = 0;
    (*pOleStreamData.offset(1)).dwMetaFileWidth = 0;
    (*pOleStreamData.offset(1)).dwMetaFileHeight = 0;
    (*pOleStreamData.offset(1)).pData = core::ptr::null_mut();
    (*pOleStreamData.offset(1)).dwDataLength = 0;

    hRes = IStorage_OpenStream(pStorage, wstrStreamName.as_ptr(), core::ptr::null_mut(), STGM_READ | STGM_SHARE_EXCLUSIVE, 0, core::ptr::addr_of_mut!(pStream));
    if hRes == 0 {
        let mut iSeekPos: LARGE_INTEGER = core::mem::zeroed();
        let mut MetaFilePict: METAFILEPICT16 = core::mem::zeroed();
        static strMetafilePictName: &[u8] = b"METAFILEPICT\0";

        (*pOleStreamData.offset(1)).dwTypeID = 5;

        (*pOleStreamData.offset(1)).dwOleTypeNameLength = strlen(strMetafilePictName.as_ptr() as *const i8) as DWORD + 1;
        strcpy((*pOleStreamData.offset(1)).strOleTypeName.as_mut_ptr() as *mut i8, strMetafilePictName.as_ptr() as *const i8);

        iSeekPos.u.HighPart = 0;
        iSeekPos.u.LowPart = core::mem::size_of_val(&olePress.byUnknown1) as u32;

        IStream_Seek(pStream, iSeekPos, 0, core::ptr::null_mut());
        IStream_Read(pStream, core::ptr::addr_of_mut!(olePress.dwExtentX) as *mut core::ffi::c_void, core::mem::size_of_val(&olePress.dwExtentX) as ULONG, core::ptr::null_mut());
        IStream_Read(pStream, core::ptr::addr_of_mut!(olePress.dwExtentY) as *mut core::ffi::c_void, core::mem::size_of_val(&olePress.dwExtentY) as ULONG, core::ptr::null_mut());
        IStream_Read(pStream, core::ptr::addr_of_mut!(olePress.dwSize) as *mut core::ffi::c_void, core::mem::size_of_val(&olePress.dwSize) as ULONG, core::ptr::null_mut());

        (*pOleStreamData.offset(1)).dwMetaFileWidth = olePress.dwExtentX;
        (*pOleStreamData.offset(1)).dwMetaFileHeight = (-(olePress.dwExtentY as i32)) as DWORD;
        if olePress.dwSize > 0 {
            (*pOleStreamData.offset(1)).dwDataLength = olePress.dwSize + core::mem::size_of::<METAFILEPICT16>() as DWORD;

            MetaFilePict.mm = 8;
            MetaFilePict.xExt = olePress.dwExtentX as i16;
            MetaFilePict.yExt = olePress.dwExtentY as i16;
            MetaFilePict.hMF = 0;

            (*pOleStreamData.offset(1)).pData = HeapAlloc(GetProcessHeap(), 0, ((*pOleStreamData.offset(1)).dwDataLength as usize) as SIZE_T) as *mut BYTE;
            memcpy((*pOleStreamData.offset(1)).pData as *mut core::ffi::c_void, core::ptr::addr_of!(MetaFilePict) as *const core::ffi::c_void, core::mem::size_of::<METAFILEPICT16>());
            IStream_Read(pStream, ((*pOleStreamData.offset(1)).pData as usize + core::mem::size_of::<METAFILEPICT16>()) as *mut core::ffi::c_void, (*pOleStreamData.offset(1)).dwDataLength - core::mem::size_of::<METAFILEPICT16>() as DWORD, core::ptr::null_mut());
        }
        IStream_Release(pStream);
    }
}

#[no_mangle]
pub unsafe extern "system" fn OleConvertOLESTREAMToIStorage(
    pOleStream: LPOLESTREAM,
    pstg: *mut IStorage,
    ptd: *const DVTARGETDEVICE,
) -> HRESULT {
    let mut i: i32;
    let mut hRes: HRESULT = S_OK;
    let mut pOleStreamData: [OLECONVERT_OLESTREAM_DATA; 2] = core::mem::zeroed();

    

    TRACE!(b"%p %p %p\n\0".as_ptr() as *const c_char);

    memset(
        pOleStreamData.as_mut_ptr() as *mut core::ffi::c_void,
        0,
        core::mem::size_of_val(&pOleStreamData),
    );

    if !ptd.is_null() {
        FIXME(b"DVTARGETDEVICE is not NULL, unhandled parameter\n\0".as_ptr() as *const c_char);
    }

    if pstg.is_null() || pOleStream.is_null() {
        hRes = E_INVALIDARG;
    }

    if hRes == S_OK {
        hRes = OLECONVERT_LoadOLE10(pOleStream, &mut pOleStreamData[0], 1);
    }

    if hRes == S_OK {
        hRes = OLECONVERT_LoadOLE10(pOleStream, &mut pOleStreamData[1], 0);
    }

    if hRes == S_OK {
        if pOleStreamData[0].dwDataLength > core::mem::size_of_val(&STORAGE_magic) as DWORD {
            if memcmp(
                pOleStreamData[0].pData as *const core::ffi::c_void,
                STORAGE_magic.as_ptr() as *const core::ffi::c_void,
                core::mem::size_of_val(&STORAGE_magic),
            ) == 0
            {
                OLECONVERT_GetOLE20FromOLE10(
                    pstg,
                    pOleStreamData[0].pData,
                    pOleStreamData[0].dwDataLength,
                );
                OLECONVERT_CreateOlePresStream(
                    pstg,
                    pOleStreamData[1].dwMetaFileWidth,
                    pOleStreamData[1].dwMetaFileHeight,
                    pOleStreamData[1].pData,
                    pOleStreamData[1].dwDataLength,
                );
            } else {
                OLECONVERT_CreateOle10NativeStream(
                    pstg,
                    pOleStreamData[0].pData,
                    pOleStreamData[0].dwDataLength,
                );
            }
        } else {
            OLECONVERT_CreateOle10NativeStream(
                pstg,
                pOleStreamData[0].pData,
                pOleStreamData[0].dwDataLength,
            );
        }

        hRes = OLECONVERT_CreateCompObjStream(
            pstg,
            pOleStreamData[0].strOleTypeName.as_ptr(),
        );
        if hRes == S_OK {
            STORAGE_CreateOleStream(pstg, 0);
        }
    }

    i = 0;
    while i < 2 {
        HeapFree(
            GetProcessHeap(),
            0,
            pOleStreamData[i as usize].pData as *mut core::ffi::c_void,
        );
        HeapFree(
            GetProcessHeap(),
            0,
            pOleStreamData[i as usize].pstrOleObjFileName as *mut core::ffi::c_void,
        );
        pOleStreamData[i as usize].pstrOleObjFileName = core::ptr::null_mut();
        i += 1;
    }

    hRes
}

#[no_mangle]
pub unsafe extern "system" fn OleConvertIStorageToOLESTREAM(
    pstg: *mut IStorage,
    pOleStream: LPOLESTREAM,
) -> HRESULT {
    let mut i: i32;
    let mut hRes: HRESULT = S_OK;
    let mut pStream: *mut IStream = core::ptr::null_mut();
    let mut pOleStreamData: [OLECONVERT_OLESTREAM_DATA; 2] = core::mem::zeroed();
    static wstrStreamName: [u16; 13] = [1, 'O' as u16, 'l' as u16, 'e' as u16, '1' as u16, '0' as u16, 'N' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 0];

    TRACE!(b"%p %p\n\0".as_ptr() as PCHAR);

    memset(pOleStreamData.as_mut_ptr() as *mut core::ffi::c_void, 0, core::mem::size_of_val(&pOleStreamData));

    if pstg.is_null() || pOleStream.is_null() {
        hRes = E_INVALIDARG;
    }
    if hRes == S_OK {
        pOleStreamData[0].dwOleTypeNameLength = OLESTREAM_MAX_STR_LEN as u32;
        hRes = OLECONVERT_GetOLE10ProgID(pstg, pOleStreamData[0].strOleTypeName.as_mut_ptr() as *mut u8, &mut pOleStreamData[0].dwOleTypeNameLength);
    }
    if hRes == S_OK {
        hRes = IStorage_OpenStream(pstg, wstrStreamName.as_ptr(), core::ptr::null_mut(), STGM_READ | STGM_SHARE_EXCLUSIVE, 0, &mut pStream);
        if hRes == S_OK {
            IStream_Release(pStream);
            OLECONVERT_GetOle10PresData(pstg, &mut pOleStreamData[0]);
        } else {
            OLECONVERT_GetOle20PresData(pstg, &mut pOleStreamData[0]);
        }

        hRes = OLECONVERT_SaveOLE10(&mut pOleStreamData[0], pOleStream);
        if hRes == S_OK {
            hRes = OLECONVERT_SaveOLE10(&mut pOleStreamData[1], pOleStream);
        }
    }

    i = 0;
    while i < 2 {
        HeapFree(GetProcessHeap(), 0, pOleStreamData[i as usize].pData as LPVOID);
        i += 1;
    }

    hRes
}

unsafe fn StorageImpl_ReadAt(
    This: *mut StorageImpl,
    offset: ULARGE_INTEGER,
    buffer: *mut core::ffi::c_void,
    size: ULONG,
    bytesRead: *mut ULONG,
) -> HRESULT {
    ILockBytes_ReadAt((*This).lockBytes, offset, buffer, size, bytesRead)
}

unsafe fn StorageImpl_WriteAt(
    This: *mut StorageImpl,
    offset: ULARGE_INTEGER,
    buffer: *const core::ffi::c_void,
    size: ULONG,
    bytesWritten: *mut ULONG,
) -> HRESULT {
    ILockBytes_WriteAt((*This).lockBytes, offset, buffer, size, bytesWritten)
}

unsafe fn StorageImpl_GetBigBlockOffset(This: *mut StorageImpl, index: ULONG) -> ULONGLONG {
    ((index + 1) as ULONGLONG) * ((*This).bigBlockSize as ULONGLONG)
}

unsafe fn StorageImpl_LockRegion(
    This: *mut StorageImpl,
    offset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
    supported: *mut BOOL,
) -> HRESULT {
    if (dwLockType & (*This).locks_supported) == 0 {
        if !supported.is_null() {
            *supported = 0;
        }
        return 0;
    }

    if !supported.is_null() {
        *supported = 1;
    }
    ILockBytes_LockRegion((*This).lockBytes, offset, cb, dwLockType)
}

unsafe fn StorageImpl_UnlockRegion(
    This: *mut StorageImpl,
    offset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD
) -> HRESULT {
    if (dwLockType & (*This).locks_supported) == 0 {
        return S_OK;
    }

    ILockBytes_UnlockRegion((*This).lockBytes, offset, cb, dwLockType)
}

unsafe extern "C" fn StorageImpl_GetFilename(iface: *mut StorageBaseImpl, result: *mut LPWSTR) -> HRESULT {
    let This = iface as *mut StorageImpl;
    let mut statstg: STATSTG = core::mem::zeroed();
    
    let hr = ILockBytes_Stat((*This).lockBytes, &mut statstg, 0);
    
    *result = statstg.pwcsName;
    
    hr
}

unsafe extern "C" fn StorageInternalImpl_Flush(iface: *mut StorageBaseImpl) -> HRESULT {
    let This = iface as *mut StorageInternalImpl;
    StorageBaseImpl_Flush((*This).parentStorage)
}

unsafe extern "C" fn StorageInternalImpl_GetFilename(iface: *mut StorageBaseImpl, result: *mut LPWSTR) -> HRESULT {
    let This = iface as *mut StorageInternalImpl;
    
    StorageBaseImpl_GetFilename((*This).parentStorage, result)
}

unsafe extern "C" fn StorageInternalImpl_CreateDirEntry(
    base: *mut StorageBaseImpl,
    newData: *const DirEntry,
    index: *mut DirRef,
) -> HRESULT {
    let This = base as *mut StorageInternalImpl;
    
    StorageBaseImpl_CreateDirEntry(
        (*This).parentStorage,
        newData,
        index,
    )
}

unsafe extern "C" fn StorageInternalImpl_WriteDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
    data: *const DirEntry
) -> HRESULT {
    let This = base as *mut StorageInternalImpl;
    StorageBaseImpl_WriteDirEntry((*This).parentStorage, index, data)
}

unsafe extern "C" fn StorageInternalImpl_ReadDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
    data: *mut DirEntry,
) -> HRESULT {
    let This = base as *mut StorageInternalImpl;
    StorageBaseImpl_ReadDirEntry((*This).parentStorage, index, data)
}

unsafe extern "C" fn StorageInternalImpl_DestroyDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
) -> HRESULT {
    let This = base as *mut StorageInternalImpl;
    StorageBaseImpl_DestroyDirEntry((*This).parentStorage, index)
}

unsafe extern "C" fn StorageInternalImpl_StreamReadAt(
    base: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *mut core::ffi::c_void,
    bytesRead: *mut ULONG,
) -> HRESULT {
    let This = base as *mut StorageInternalImpl;
    
    StorageBaseImpl_StreamReadAt(
        (*This).parentStorage,
        index,
        offset,
        size,
        buffer,
        bytesRead,
    )
}

unsafe extern "C" fn StorageInternalImpl_StreamWriteAt(
    base: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *const core::ffi::c_void,
    bytesWritten: *mut ULONG
) -> HRESULT {
    let This = base as *mut StorageInternalImpl;
    
    StorageBaseImpl_StreamWriteAt(
        (*This).parentStorage,
        index,
        offset,
        size,
        buffer,
        bytesWritten
    )
}

unsafe extern "C" fn StorageInternalImpl_StreamSetSize(
    base: *mut StorageBaseImpl,
    index: DirRef,
    newsize: ULARGE_INTEGER,
) -> HRESULT {
    let This = base as *mut StorageInternalImpl;
    
    StorageBaseImpl_StreamSetSize(
        (*This).parentStorage,
        index,
        newsize,
    )
}

unsafe extern "C" fn StorageInternalImpl_StreamLink(
    base: *mut StorageBaseImpl,
    dst: DirRef,
    src: DirRef,
) -> HRESULT {
    let This = base as *mut StorageInternalImpl;
    StorageBaseImpl_StreamLink((*This).parentStorage, dst, src)
}

unsafe extern "C" fn StorageInternalImpl_GetTransactionSig(
    _base: *mut StorageBaseImpl,
    _result: *mut ULONG,
    _refresh: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn StorageInternalImpl_SetTransactionSig(
    _base: *mut StorageBaseImpl,
    _value: ULONG,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn StorageInternalImpl_LockTransaction(_base: *mut StorageBaseImpl, _write: BOOL) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn StorageInternalImpl_UnlockTransaction(
    _base: *mut StorageBaseImpl,
    _write: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

stdmethod! {
fn StorageInternalImpl_Revert(
    iface: *mut IStorage
) -> HRESULT {
    FIXME(b"(%p): stub\n\0".as_ptr() as PCHAR);
    S_OK
}
}

unsafe fn StorageInternalImpl_Construct(
    parentStorage: *mut StorageBaseImpl,
    openFlags: DWORD,
    storageDirEntry: DirRef,
) -> *mut StorageInternalImpl {
    let newStorage: *mut StorageInternalImpl;

    newStorage = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<StorageInternalImpl>()) as SIZE_T,
    ) as *mut StorageInternalImpl;

    if !newStorage.is_null() {
        list_init(core::ptr::addr_of_mut!((*newStorage).base.strmHead));

        list_init(core::ptr::addr_of_mut!((*newStorage).base.storageHead));

        (*newStorage).base.IStorage_iface.lpVtbl = &StorageInternalImpl_Vtbl as *const IStorageVtbl as *mut IStorageVtbl;
        (*newStorage).base.IPropertySetStorage_iface.lpVtbl = &IPropertySetStorage_Vtbl as *const IPropertySetStorageVtbl as *mut IPropertySetStorageVtbl;
        (*newStorage).base.baseVtbl = &StorageInternalImpl_BaseVtbl;
        (*newStorage).base.openFlags = openFlags & !STGM_CREATE;

        (*newStorage).base.reverted = 0;

        (*newStorage).base.ref_ = 1;

        (*newStorage).parentStorage = parentStorage;

        (*newStorage).base.storageDirEntry = storageDirEntry;

        (*newStorage).base.create = 0;

        return newStorage;
    }

    core::ptr::null_mut()
}

unsafe fn TransactedSnapshotImpl_FindFreeEntry(This: *mut TransactedSnapshotImpl) -> DirRef {
    let mut result: DirRef = (*This).firstFreeEntry;

    while result < (*This).entries_size && (*(*This).entries.offset(result as isize)).inuse != 0 {
        result = result.wrapping_add(1);
    }

    if result == (*This).entries_size {
        let new_size: ULONG = (*This).entries_size.wrapping_mul(2);
        let new_entries: *mut TransactedDirEntry;

        new_entries = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((core::mem::size_of::<TransactedDirEntry>()).wrapping_mul(new_size as usize)) as SIZE_T
        ) as *mut TransactedDirEntry;
        if new_entries.is_null() {
            return DIRENTRY_NULL;
        }

        memcpy(
            new_entries as *mut core::ffi::c_void,
            (*This).entries as *const core::ffi::c_void,
            (core::mem::size_of::<TransactedDirEntry>()).wrapping_mul((*This).entries_size as usize)
        );
        HeapFree(GetProcessHeap(), 0, (*This).entries as *mut core::ffi::c_void);

        (*This).entries = new_entries;
        (*This).entries_size = new_size;
    }

    (*(*This).entries.offset(result as isize)).inuse = 1;

    (*This).firstFreeEntry = result.wrapping_add(1);

    result
}

unsafe fn TransactedSnapshotImpl_FindFirstChild(
    This: *mut TransactedSnapshotImpl,
    parent: DirRef,
) -> DirRef {
    let mut cursor: DirRef;
    let mut prev: DirRef;
    let mut entry: *mut TransactedDirEntry;

    cursor = parent;
    entry = (*This).entries.offset(cursor as isize);
    
    while (*entry).read != 0 {
        if (*entry).data.leftChild != DIRENTRY_NULL {
            prev = cursor;
            cursor = (*entry).data.leftChild;
            entry = (*This).entries.offset(cursor as isize);
            (*entry).parent = prev;
        } else if (*entry).data.rightChild != DIRENTRY_NULL {
            prev = cursor;
            cursor = (*entry).data.rightChild;
            entry = (*This).entries.offset(cursor as isize);
            (*entry).parent = prev;
        } else if (*entry).data.dirRootEntry != DIRENTRY_NULL {
            prev = cursor;
            cursor = (*entry).data.dirRootEntry;
            entry = (*This).entries.offset(cursor as isize);
            (*entry).parent = prev;
        } else {
            break;
        }
    }

    cursor
}

unsafe fn TransactedSnapshotImpl_MadeCopy(
    This: *mut TransactedSnapshotImpl,
    entry: DirRef,
) -> BOOL {
    (entry != DIRENTRY_NULL
        && (*This).entries.offset(entry as isize).read().newTransactedParentEntry
            != (*This).entries.offset(entry as isize).read().transactedParentEntry) as BOOL
}

unsafe extern "C" fn TransactedSnapshotImpl_Destroy(iface: *mut StorageBaseImpl) {
    let This = iface as *mut TransactedSnapshotImpl;
    
    IStorage_Revert(&mut (*This).base.IStorage_iface);
    IStorage_Release(&mut (*(*This).transactedParent).IStorage_iface);
    IStorage_Release(&mut (*(*This).scratch).IStorage_iface);
    HeapFree(GetProcessHeap(), 0, (*This).entries as LPVOID);
    HeapFree(GetProcessHeap(), 0, This as LPVOID);
}

unsafe extern "C" fn TransactedSnapshotImpl_Flush(_iface: *mut StorageBaseImpl) -> HRESULT {
    S_OK
}

unsafe extern "C" fn TransactedSnapshotImpl_GetFilename(iface: *mut StorageBaseImpl, result: *mut LPWSTR) -> HRESULT {
    let This = iface as *mut TransactedSnapshotImpl;
    
    StorageBaseImpl_GetFilename((*This).transactedParent, result)
}

unsafe extern "C" fn TransactedSnapshotImpl_DestroyDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
) -> HRESULT {
    let This = base as *mut TransactedSnapshotImpl;

    if (*This).entries.offset(index as isize).read().transactedParentEntry == DIRENTRY_NULL
        || (*This).entries.offset(index as isize).read().data.size.QuadPart != 0
    {
        core::ptr::write_bytes(
            (*This).entries.offset(index as isize),
            0,
            1,
        );
        (*This).firstFreeEntry = if index < (*This).firstFreeEntry {
            index
        } else {
            (*This).firstFreeEntry
        };
    } else {
        (*(*This).entries.offset(index as isize)).deleted = 1;
    }

    S_OK
}

unsafe extern "C" fn TransactedSnapshotImpl_StreamReadAt(
    base: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *mut core::ffi::c_void,
    bytesRead: *mut ULONG,
) -> HRESULT {
    let This = base as *mut TransactedSnapshotImpl;

    if (*(*This).entries.offset(index as isize)).stream_dirty != 0 {
        return StorageBaseImpl_StreamReadAt(
            (*This).scratch,
            (*(*This).entries.offset(index as isize)).stream_entry,
            offset,
            size,
            buffer,
            bytesRead,
        );
    } else if (*(*This).entries.offset(index as isize)).transactedParentEntry == DIRENTRY_NULL {
        *bytesRead = 0;
        return S_OK;
    } else {
        return StorageBaseImpl_StreamReadAt(
            (*This).transactedParent,
            (*(*This).entries.offset(index as isize)).transactedParentEntry,
            offset,
            size,
            buffer,
            bytesRead,
        );
    }
}

unsafe extern "C" fn TransactedSnapshotImpl_GetTransactionSig(
    _base: *mut StorageBaseImpl,
    _result: *mut ULONG,
    _refresh: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn TransactedSnapshotImpl_SetTransactionSig(
    _base: *mut StorageBaseImpl,
    _value: ULONG,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn TransactedSnapshotImpl_LockTransaction(_base: *mut StorageBaseImpl, _write: BOOL) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn TransactedSnapshotImpl_UnlockTransaction(_base: *mut StorageBaseImpl, _write: BOOL) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn TransactedSharedImpl_Flush(_iface: *mut StorageBaseImpl) -> HRESULT {
    S_OK
}

unsafe extern "C" fn TransactedSharedImpl_GetFilename(iface: *mut StorageBaseImpl, result: *mut LPWSTR) -> HRESULT {
    let This = iface as *mut TransactedSharedImpl;
    StorageBaseImpl_GetFilename((*This).transactedParent, result)
}

unsafe extern "C" fn TransactedSharedImpl_CreateDirEntry(
    base: *mut StorageBaseImpl,
    newData: *const DirEntry,
    index: *mut DirRef,
) -> HRESULT {
    let This = base as *mut TransactedSharedImpl;
    StorageBaseImpl_CreateDirEntry(
        &mut (*(*This).scratch).base,
        newData,
        index,
    )
}

unsafe extern "C" fn TransactedSharedImpl_WriteDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
    data: *const DirEntry,
) -> HRESULT {
    let This = base as *mut TransactedSharedImpl;
    
    StorageBaseImpl_WriteDirEntry(
        &mut (*(*This).scratch).base,
        index,
        data,
    )
}

unsafe extern "C" fn TransactedSharedImpl_ReadDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
    data: *mut DirEntry,
) -> HRESULT {
    let This = base as *mut TransactedSharedImpl;
    StorageBaseImpl_ReadDirEntry(
        core::ptr::addr_of_mut!((*(*This).scratch).base),
        index,
        data,
    )
}

unsafe extern "C" fn TransactedSharedImpl_DestroyDirEntry(
    base: *mut StorageBaseImpl,
    index: DirRef,
) -> HRESULT {
    let This = base as *mut TransactedSharedImpl;
    
    StorageBaseImpl_DestroyDirEntry(
        &mut (*(*This).scratch).base,
        index,
    )
}

unsafe extern "C" fn TransactedSharedImpl_StreamReadAt(
    base: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *mut core::ffi::c_void,
    bytesRead: *mut ULONG,
) -> HRESULT {
    let This = base as *mut TransactedSharedImpl;
    
    StorageBaseImpl_StreamReadAt(
        core::ptr::addr_of_mut!((*(*This).scratch).base),
        index,
        offset,
        size,
        buffer,
        bytesRead,
    )
}

unsafe extern "C" fn TransactedSharedImpl_StreamWriteAt(
    base: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *const core::ffi::c_void,
    bytesWritten: *mut ULONG,
) -> HRESULT {
    let This = base as *mut TransactedSharedImpl;
    
    StorageBaseImpl_StreamWriteAt(
        core::ptr::addr_of_mut!((*(*This).scratch).base),
        index,
        offset,
        size,
        buffer,
        bytesWritten,
    )
}

unsafe extern "C" fn TransactedSharedImpl_StreamSetSize(
    base: *mut StorageBaseImpl,
    index: DirRef,
    newsize: ULARGE_INTEGER,
) -> HRESULT {
    let This = base as *mut TransactedSharedImpl;
    StorageBaseImpl_StreamSetSize(
        &mut (*(*This).scratch).base,
        index,
        newsize,
    )
}

unsafe extern "C" fn TransactedSharedImpl_StreamLink(
    base: *mut StorageBaseImpl,
    dst: DirRef,
    src: DirRef,
) -> HRESULT {
    let This = base as *mut TransactedSharedImpl;
    StorageBaseImpl_StreamLink(
        &mut (*(*This).scratch).base,
        dst,
        src,
    )
}

unsafe extern "C" fn TransactedSharedImpl_GetTransactionSig(
    _base: *mut StorageBaseImpl,
    _result: *mut ULONG,
    _refresh: BOOL,
) -> HRESULT {
    0x80004001u32 as HRESULT
}

unsafe extern "C" fn TransactedSharedImpl_SetTransactionSig(
    _base: *mut StorageBaseImpl,
    _value: ULONG,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn TransactedSharedImpl_LockTransaction(_base: *mut StorageBaseImpl, _write: BOOL) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn TransactedSharedImpl_UnlockTransaction(
    _base: *mut StorageBaseImpl,
    _write: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn StorageUtl_ReadWord(buffer: *const u8, offset: u32, value: *mut u16) {
    let mut tmp: u16 = 0;
    
    core::ptr::copy_nonoverlapping(
        buffer.add(offset as usize),
        &mut tmp as *mut u16 as *mut u8,
        core::mem::size_of::<u16>()
    );
    *value = u16::from_le(tmp);
}

pub unsafe fn StorageUtl_WriteWord(buffer: *mut BYTE, offset: ULONG, value: u16) {
    
    
    let value = htole16(value);
    memcpy(
        buffer.offset(offset as isize) as *mut core::ffi::c_void,
        &value as *const u16 as *const core::ffi::c_void,
        core::mem::size_of::<u16>()
    );
}

pub unsafe fn StorageUtl_ReadDWord(buffer: *const BYTE, offset: ULONG, value: *mut DWORD) {
    let mut tmp: DWORD = 0;
    
    
    
    memcpy(
        core::ptr::addr_of_mut!(tmp) as *mut core::ffi::c_void,
        buffer.offset(offset as isize) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>()
    );
    *value = lendian32toh(tmp);
}

pub unsafe fn StorageUtl_WriteDWord(buffer: *mut BYTE, offset: ULONG, value: DWORD) {
    let value = htole32(value);
    core::ptr::copy_nonoverlapping(
        &value as *const DWORD as *const u8,
        buffer.offset(offset as isize),
        core::mem::size_of::<DWORD>()
    );
}



pub unsafe fn StorageUtl_ReadULargeInteger(
    buffer: *const BYTE,
    offset: ULONG,
    value: *mut ULARGE_INTEGER,
) {
    #[cfg(target_endian = "big")]
    {
        
        let mut tmp: ULARGE_INTEGER = core::mem::zeroed();
        core::ptr::copy_nonoverlapping(
            buffer.add(offset as usize),
            &mut tmp as *mut ULARGE_INTEGER as *mut BYTE,
            core::mem::size_of::<ULARGE_INTEGER>(),
        );
        (*value).u.LowPart = htole32(tmp.u.HighPart);
        (*value).u.HighPart = htole32(tmp.u.LowPart);
    }
    #[cfg(not(target_endian = "big"))]
    {
        core::ptr::copy_nonoverlapping(
            buffer.add(offset as usize),
            value as *mut BYTE,
            core::mem::size_of::<ULARGE_INTEGER>(),
        );
    }
}

pub unsafe fn StorageUtl_WriteULargeInteger(
    buffer: *mut u8,
    offset: u32,
    value: *const ULARGE_INTEGER,
) {
    #[cfg(target_endian = "big")]
    {
        
        
        let mut tmp: ULARGE_INTEGER = core::mem::zeroed();
        tmp.u.LowPart = htole32((*value).u.HighPart);
        tmp.u.HighPart = htole32((*value).u.LowPart);
        core::ptr::copy_nonoverlapping(
            &tmp as *const ULARGE_INTEGER as *const u8,
            buffer.offset(offset as isize),
            core::mem::size_of::<ULARGE_INTEGER>(),
        );
    }
    #[cfg(not(target_endian = "big"))]
    {
        core::ptr::copy_nonoverlapping(
            value as *const u8,
            buffer.offset(offset as isize),
            core::mem::size_of::<ULARGE_INTEGER>(),
        );
    }
}

pub unsafe fn StorageUtl_CopyDirEntryToSTATSTG(
    storage: *mut StorageBaseImpl,
    destination: *mut STATSTG,
    source: *const DirEntry,
    statFlags: i32,
) {
    if (statFlags & STATFLAG_NONAME as c_int) == 0 && (*source).stgType as DWORD == STGTY_ROOT {
        (*destination).pwcsName = core::ptr::null_mut();
        StorageBaseImpl_GetFilename(storage, core::ptr::addr_of_mut!((*destination).pwcsName));
    } else if (statFlags & STATFLAG_NONAME as c_int) != 0 || (*source).name[0] == 0 {
        (*destination).pwcsName = core::ptr::null_mut();
    } else {
        (*destination).pwcsName = CoTaskMemAlloc(
            ((lstrlenW((*source).name.as_ptr()) as usize + 1) * core::mem::size_of::<u16>()) as SIZE_T
        ) as *mut u16;
        lstrcpyW((*destination).pwcsName, (*source).name.as_ptr());
    }

    match (*source).stgType as DWORD {
        STGTY_STORAGE | STGTY_ROOT => {
            (*destination).type_ = STGTY_STORAGE;
        }
        STGTY_STREAM => {
            (*destination).type_ = STGTY_STREAM;
        }
        _ => {
            (*destination).type_ = STGTY_STREAM;
        }
    }

    (*destination).cbSize = (*source).size;
    (*destination).grfMode = 0;
    (*destination).grfLocksSupported = 0;
    (*destination).clsid = (*source).clsid;
    (*destination).grfStateBits = 0;
    (*destination).reserved = 0;
}



unsafe fn BlockChainStream_GetSectorOfOffset(This: *mut BlockChainStream, offset: ULONG) -> ULONG {
    let mut min_offset: ULONG = 0;
    let mut max_offset: ULONG = (*This).numBlocks.wrapping_sub(1);
    let mut min_run: ULONG = 0;
    let mut max_run: ULONG = (*This).indexCacheLen.wrapping_sub(1);

    if offset >= (*This).numBlocks {
        return BLOCK_END_OF_CHAIN;
    }

    while min_run < max_run {
        let run_to_check: ULONG = min_run
            .wrapping_add(
                offset
                    .wrapping_sub(min_offset)
                    .wrapping_mul(max_run.wrapping_sub(min_run))
                    / max_offset.wrapping_sub(min_offset),
            );
        if offset < (*(*This).indexCache.offset(run_to_check as isize)).firstOffset {
            max_offset = (*(*This).indexCache.offset(run_to_check as isize))
                .firstOffset
                .wrapping_sub(1);
            max_run = run_to_check.wrapping_sub(1);
        } else if offset > (*(*This).indexCache.offset(run_to_check as isize)).lastOffset {
            min_offset = (*(*This).indexCache.offset(run_to_check as isize))
                .lastOffset
                .wrapping_add(1);
            min_run = run_to_check.wrapping_add(1);
        } else {
            min_run = run_to_check;
            max_run = run_to_check;
        }
    }

    (*(*This).indexCache.offset(min_run as isize))
        .firstSector
        .wrapping_add(offset)
        .wrapping_sub((*(*This).indexCache.offset(min_run as isize)).firstOffset)
}

unsafe fn SmallBlockChainStream_Construct(
    parentStorage: *mut StorageImpl,
    headOfStreamPlaceHolder: *mut ULONG,
    dirEntry: DirRef,
) -> *mut SmallBlockChainStream {
    

    let newStream = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<SmallBlockChainStream>()) as SIZE_T,
    ) as *mut SmallBlockChainStream;

    (*newStream).parentStorage = parentStorage;
    (*newStream).headOfStreamPlaceHolder = headOfStreamPlaceHolder;
    (*newStream).ownerDirEntry = dirEntry;

    newStream
}

unsafe fn SmallBlockChainStream_Destroy(This: *mut SmallBlockChainStream) {
    
    
    HeapFree(GetProcessHeap(), 0, This as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn StgCreatePropSetStg(
    pstg: *mut IStorage,
    reserved: DWORD,
    propset: *mut *mut IPropertySetStorage,
) -> HRESULT {
    

    TRACE!(b"(%p, 0x%x, %p)\n\0".as_ptr() as *const c_char);

    if reserved != 0 {
        return STG_E_INVALIDPARAMETER;
    }

    IStorage_QueryInterface(
        pstg,
        &IID_IPropertySetStorage,
        propset as *mut *mut core::ffi::c_void,
    )
}

#[no_mangle]
pub unsafe extern "system" fn StgIsStorageILockBytes(plkbyt: *mut ILockBytes) -> HRESULT {
    
    

    let mut sig: [BYTE; 8] = [0; 8];
    let mut offset: ULARGE_INTEGER = core::mem::zeroed();
    let mut read: ULONG = 0;

    offset.u.HighPart = 0;
    offset.u.LowPart = 0;

    ILockBytes_ReadAt(
        plkbyt,
        offset,
        sig.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&sig) as ULONG,
        &mut read,
    );

    if read == core::mem::size_of_val(&sig) as ULONG
        && memcmp(
            sig.as_ptr() as *const core::ffi::c_void,
            STORAGE_magic.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&sig),
        ) == 0
    {
        return 0; // S_OK
    }

    1 // S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn WriteClassStg(pStg: *mut IStorage, rclsid: *const CLSID) -> HRESULT {
    if pStg.is_null() {
        return E_INVALIDARG;
    }

    if rclsid.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    IStorage_SetClass(pStg, rclsid)
}

#[no_mangle]
pub unsafe extern "system" fn ReadClassStg(pstg: *mut IStorage, pclsid: *mut CLSID) -> HRESULT {
    

    let mut pstatstg: STATSTG = core::mem::zeroed();
    let hRes: HRESULT;

    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const c_char);

    if pstg.is_null() || pclsid.is_null() {
        return 0x80070057u32 as HRESULT;
    }

    hRes = IStorage_Stat(pstg, &mut pstatstg, 1);

    if SUCCEEDED(hRes) {
        *pclsid = pstatstg.clsid;
    }

    return hRes;
}

// storage32.h declares this WITHOUT `WINAPI`, so it is CDECL, not stdcall. It is
// DECLSPEC_HIDDEN (not an ole32 export), and ring/clipboard.rs declares it `extern "C"`
// — an `extern "system"` definition emits `_Name@N` on i386 and leaves that decl
// dangling. Caught by scan_cc_mismatch.
#[no_mangle]
pub unsafe extern "C" fn STORAGE_CreateOleStream(
    storage: *mut IStorage,
    flags: DWORD,
) -> HRESULT {
    static stream_1oleW: [WCHAR; 5] = [1, 'O' as u16, 'l' as u16, 'e' as u16, 0];
    const version_magic: DWORD = 0x02000001;
    
    let mut stream: *mut IStream = core::ptr::null_mut();
    
    let hr = ((*(*storage).lpVtbl).CreateStream.unwrap_unchecked())(
        storage,
        stream_1oleW.as_ptr(),
        STGM_WRITE | STGM_SHARE_EXCLUSIVE,
        0,
        0,
        &mut stream,
    );
    
    if hr == S_OK {
        #[repr(C)]
        struct empty_1ole_stream {
            version_magic: DWORD,
            flags: DWORD,
            update_options: DWORD,
            reserved: DWORD,
            mon_stream_size: DWORD,
        }
        
        let stream_data = empty_1ole_stream {
            version_magic,
            flags,
            update_options: 0,
            reserved: 0,
            mon_stream_size: 0,
        };
        
        let hr = ((*(*stream).lpVtbl).Write.unwrap_unchecked())(
            stream,
            &stream_data as *const _ as *const core::ffi::c_void,
            core::mem::size_of::<empty_1ole_stream>() as ULONG,
            core::ptr::null_mut(),
        );
        
        ((*(*stream).lpVtbl).Release.unwrap_unchecked())(stream);
        
        return hr;
    }
    
    hr
}

unsafe fn STREAM_WriteString(stm: *mut IStream, string: LPCWSTR) -> HRESULT {
    let mut r: HRESULT;
    let str: LPSTR;
    let mut len: DWORD = 0;

    if !string.is_null() {
        len = WideCharToMultiByte(CP_ACP, 0, string, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as DWORD;
    }
    r = IStream_Write(stm, &len as *const _ as *const core::ffi::c_void, core::mem::size_of::<DWORD>() as ULONG, core::ptr::null_mut());
    if FAILED(r) {
        return r;
    }
    if len == 0 {
        return r;
    }
    str = CoTaskMemAlloc((len as usize) as SIZE_T) as LPSTR;
    WideCharToMultiByte(CP_ACP, 0, string, -1, str, len as c_int, core::ptr::null(), core::ptr::null_mut());
    r = IStream_Write(stm, str as *const core::ffi::c_void, len, core::ptr::null_mut());
    CoTaskMemFree(str as LPVOID);
    r
}

unsafe fn STREAM_ReadString(stm: *mut IStream, string: *mut LPWSTR) -> HRESULT {
    let mut len: DWORD = 0;
    let mut count: DWORD = 0;
    let str: LPSTR;
    let wstr: LPWSTR;

    let mut r = IStream_Read(stm, &mut len as *mut _ as LPVOID, core::mem::size_of::<DWORD>() as ULONG, &mut count);
    if FAILED(r) {
        return r;
    }
    if count != core::mem::size_of::<DWORD>() as DWORD {
        return E_OUTOFMEMORY;
    }

    TRACE!(b"%d bytes\n\0".as_ptr() as LPCSTR);

    str = CoTaskMemAlloc((len as usize) as SIZE_T) as LPSTR;
    if str.is_null() {
        return E_OUTOFMEMORY;
    }
    count = 0;
    r = IStream_Read(stm, str as LPVOID, len, &mut count);
    if FAILED(r) {
        CoTaskMemFree(str as LPVOID);
        return r;
    }
    if count != len {
        CoTaskMemFree(str as LPVOID);
        return E_OUTOFMEMORY;
    }

    TRACE!(b"Read string %s\n\0".as_ptr() as LPCSTR);

    len = MultiByteToWideChar(CP_ACP, 0, str, count as i32, core::ptr::null_mut(), 0) as DWORD;
    wstr = CoTaskMemAlloc((((len + 1) * core::mem::size_of::<WCHAR>() as DWORD) as usize) as SIZE_T) as LPWSTR;
    if !wstr.is_null() {
        MultiByteToWideChar(CP_ACP, 0, str, count as i32, wstr, len as i32);
        *wstr.offset(len as isize) = 0;
    }
    CoTaskMemFree(str as LPVOID);

    *string = wstr;

    return r;
}

#[no_mangle]
pub unsafe extern "system" fn StgIsStorageFile(fn_: LPCOLESTR) -> HRESULT {
    let hf: *mut core::ffi::c_void;
    let mut magic: [u8; 8] = [0; 8];
    let mut bytes_read: u32 = 0;

    

    TRACE!(b"%s\n\0".as_ptr() as *const c_char);
    
    hf = CreateFileW(
        fn_,
        GENERIC_READ,
        FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    if hf == INVALID_HANDLE_VALUE {
        return 0x80030002u32 as HRESULT;
    }

    if ReadFile(
        hf,
        magic.as_mut_ptr() as *mut core::ffi::c_void,
        8,
        &mut bytes_read,
        core::ptr::null_mut(),
    ) == 0
    {
        WARN!(b" unable to read file\n\0".as_ptr() as *const c_char);
        CloseHandle(hf);
        return 0x00000001u32 as HRESULT;
    }

    CloseHandle(hf);

    if bytes_read != 8 {
        TRACE!(b" too short\n\0".as_ptr() as *const c_char);
        return 0x00000001u32 as HRESULT;
    }

    if memcmp(
        magic.as_ptr() as *const core::ffi::c_void,
        STORAGE_magic.as_ptr() as *const core::ffi::c_void,
        8,
    ) == 0
    {
        TRACE!(b" -> YES\n\0".as_ptr() as *const c_char);
        return 0x00000000u32 as HRESULT;
    }

    TRACE!(b" -> Invalid header.\n\0".as_ptr() as *const c_char);
    0x00000001u32 as HRESULT
}

#[no_mangle]
pub unsafe extern "system" fn WriteClassStm(pStm: *mut IStream, rclsid: *const CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as LPCSTR);

    if pStm.is_null() || rclsid.is_null() {
        return E_INVALIDARG;
    }

    IStream_Write(pStm, rclsid as *const core::ffi::c_void, core::mem::size_of::<CLSID>() as ULONG, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn ReadClassStm(pStm: *mut IStream, pclsid: *mut CLSID) -> HRESULT {
    let mut nbByte: ULONG = 0;
    let res: HRESULT;

    

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char);

    if pStm.is_null() || pclsid.is_null() {
        return E_INVALIDARG;
    }

    core::ptr::write_bytes(pclsid, 0, 1);

    res = IStream_Read(pStm, pclsid as *mut core::ffi::c_void, core::mem::size_of::<CLSID>() as ULONG, &mut nbByte);

    if FAILED(res) {
        return res;
    }

    if nbByte != core::mem::size_of::<CLSID>() as ULONG {
        return STG_E_READFAULT;
    } else {
        return S_OK;
    }
}

unsafe fn OLECONVERT_LoadOLE10(
    pOleStream: LPOLESTREAM,
    pData: *mut OLECONVERT_OLESTREAM_DATA,
    bStrem1: BOOL,
) -> HRESULT {
    let mut dwSize: DWORD;
    let mut hRes: HRESULT = S_OK;
    let mut nTryCnt: i32 = 0;
    let max_try: i32 = 6;

    (*pData).pData = core::ptr::null_mut();
    (*pData).pstrOleObjFileName = core::ptr::null_mut();

    while nTryCnt < max_try {
        dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
            pOleStream,
            core::ptr::addr_of_mut!((*pData).dwOleID) as *mut i8,
            (core::mem::size_of::<DWORD>()) as DWORD,
        );
        if dwSize != core::mem::size_of::<DWORD>() as DWORD {
            hRes = CONVERT10_E_OLESTREAM_GET;
        } else if (*pData).dwOleID != OLESTREAM_ID {
            hRes = CONVERT10_E_OLESTREAM_FMT;
        } else {
            hRes = S_OK;
            break;
        }
        nTryCnt += 1;
    }

    if hRes == S_OK {
        dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
            pOleStream,
            core::ptr::addr_of_mut!((*pData).dwTypeID) as *mut i8,
            (core::mem::size_of::<DWORD>()) as DWORD,
        );
        if dwSize != core::mem::size_of::<DWORD>() as DWORD {
            hRes = CONVERT10_E_OLESTREAM_GET;
        }
    }

    if hRes == S_OK {
        if (*pData).dwTypeID != 0 {
            dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
                pOleStream,
                core::ptr::addr_of_mut!((*pData).dwOleTypeNameLength) as *mut i8,
                (core::mem::size_of::<DWORD>()) as DWORD,
            );
            if dwSize != core::mem::size_of::<DWORD>() as DWORD {
                hRes = CONVERT10_E_OLESTREAM_GET;
            }

            if hRes == S_OK {
                if (*pData).dwOleTypeNameLength > 0 {
                    dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
                        pOleStream,
                        (*pData).strOleTypeName.as_mut_ptr() as *mut i8,
                        ((*pData).dwOleTypeNameLength) as DWORD,
                    );
                    if dwSize != (*pData).dwOleTypeNameLength {
                        hRes = CONVERT10_E_OLESTREAM_GET;
                    }
                }
            }

            if bStrem1 != 0 {
                dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
                    pOleStream,
                    core::ptr::addr_of_mut!((*pData).dwOleObjFileNameLength) as *mut i8,
                    (core::mem::size_of::<DWORD>()) as DWORD,
                );
                if dwSize != core::mem::size_of::<DWORD>() as DWORD {
                    hRes = CONVERT10_E_OLESTREAM_GET;
                }
                if hRes == S_OK {
                    if (*pData).dwOleObjFileNameLength < 1 {
                        (*pData).dwOleObjFileNameLength = core::mem::size_of::<DWORD>() as DWORD;
                    }
                    (*pData).pstrOleObjFileName = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        ((*pData).dwOleObjFileNameLength as usize) as SIZE_T,
                    ) as *mut CHAR;
                    if !(*pData).pstrOleObjFileName.is_null() {
                        dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
                            pOleStream,
                            (*pData).pstrOleObjFileName as *mut i8,
                            ((*pData).dwOleObjFileNameLength) as DWORD,
                        );
                        if dwSize != (*pData).dwOleObjFileNameLength {
                            hRes = CONVERT10_E_OLESTREAM_GET;
                        }
                    } else {
                        hRes = CONVERT10_E_OLESTREAM_GET;
                    }
                }
            } else {
                dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
                    pOleStream,
                    core::ptr::addr_of_mut!((*pData).dwMetaFileWidth) as *mut i8,
                    (core::mem::size_of::<DWORD>()) as DWORD,
                );
                if dwSize != core::mem::size_of::<DWORD>() as DWORD {
                    hRes = CONVERT10_E_OLESTREAM_GET;
                }
                if hRes == S_OK {
                    dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
                        pOleStream,
                        core::ptr::addr_of_mut!((*pData).dwMetaFileHeight) as *mut i8,
                        (core::mem::size_of::<DWORD>()) as DWORD,
                    );
                    if dwSize != core::mem::size_of::<DWORD>() as DWORD {
                        hRes = CONVERT10_E_OLESTREAM_GET;
                    }
                }
            }

            if hRes == S_OK {
                dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
                    pOleStream,
                    core::ptr::addr_of_mut!((*pData).dwDataLength) as *mut i8,
                    (core::mem::size_of::<DWORD>()) as DWORD,
                );
                if dwSize != core::mem::size_of::<DWORD>() as DWORD {
                    hRes = CONVERT10_E_OLESTREAM_GET;
                }
            }

            if hRes == S_OK {
                if bStrem1 == 0 {
                    (*pData).dwDataLength -= 8;
                    dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
                        pOleStream,
                        (*pData).strUnknown.as_mut_ptr() as *mut i8,
                        (core::mem::size_of_val(&(*pData).strUnknown)) as DWORD,
                    );
                    if dwSize != core::mem::size_of_val(&(*pData).strUnknown) as DWORD {
                        hRes = CONVERT10_E_OLESTREAM_GET;
                    }
                }
            }

            if hRes == S_OK {
                if (*pData).dwDataLength > 0 {
                    (*pData).pData = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        ((*pData).dwDataLength as usize) as SIZE_T,
                    ) as *mut BYTE;

                    if !(*pData).pData.is_null() {
                        dwSize = ((*(*pOleStream).lpstbl).Get.unwrap_unchecked())(
                            pOleStream,
                            (*pData).pData as *mut i8,
                            ((*pData).dwDataLength) as DWORD,
                        );
                        if dwSize != (*pData).dwDataLength {
                            hRes = CONVERT10_E_OLESTREAM_GET;
                        }
                    } else {
                        hRes = CONVERT10_E_OLESTREAM_GET;
                    }
                }
            }
        }
    }

    hRes
}

unsafe fn OLECONVERT_SaveOLE10(pData: *mut OLECONVERT_OLESTREAM_DATA, pOleStream: LPOLESTREAM) -> HRESULT {
    let mut dwSize: DWORD;
    let mut hRes: HRESULT = S_OK;

    dwSize = ((*(*pOleStream).lpstbl).Put.unwrap_unchecked())(
        pOleStream,
        core::ptr::addr_of_mut!((*pData).dwOleID) as *mut i8,
        core::mem::size_of::<DWORD>() as DWORD,
    );
    if dwSize != core::mem::size_of::<DWORD>() as DWORD {
        hRes = CONVERT10_E_OLESTREAM_PUT;
    }

    if hRes == S_OK {
        dwSize = ((*(*pOleStream).lpstbl).Put.unwrap_unchecked())(
            pOleStream,
            core::ptr::addr_of_mut!((*pData).dwTypeID) as *mut i8,
            core::mem::size_of::<DWORD>() as DWORD,
        );
        if dwSize != core::mem::size_of::<DWORD>() as DWORD {
            hRes = CONVERT10_E_OLESTREAM_PUT;
        }
    }

    if (*pData).dwOleID == OLESTREAM_ID && (*pData).dwTypeID != 0 && hRes == S_OK {
        dwSize = ((*(*pOleStream).lpstbl).Put.unwrap_unchecked())(
            pOleStream,
            core::ptr::addr_of_mut!((*pData).dwOleTypeNameLength) as *mut i8,
            core::mem::size_of::<DWORD>() as DWORD,
        );
        if dwSize != core::mem::size_of::<DWORD>() as DWORD {
            hRes = CONVERT10_E_OLESTREAM_PUT;
        }

        if hRes == S_OK {
            if (*pData).dwOleTypeNameLength > 0 {
                dwSize = ((*(*pOleStream).lpstbl).Put.unwrap_unchecked())(
                    pOleStream,
                    (*pData).strOleTypeName.as_mut_ptr() as *mut i8,
                    ((*pData).dwOleTypeNameLength) as DWORD,
                );
                if dwSize != (*pData).dwOleTypeNameLength {
                    hRes = CONVERT10_E_OLESTREAM_PUT;
                }
            }
        }

        if hRes == S_OK {
            dwSize = ((*(*pOleStream).lpstbl).Put.unwrap_unchecked())(
                pOleStream,
                core::ptr::addr_of_mut!((*pData).dwMetaFileWidth) as *mut i8,
                core::mem::size_of::<DWORD>() as DWORD,
            );
            if dwSize != core::mem::size_of::<DWORD>() as DWORD {
                hRes = CONVERT10_E_OLESTREAM_PUT;
            }
        }

        if hRes == S_OK {
            dwSize = ((*(*pOleStream).lpstbl).Put.unwrap_unchecked())(
                pOleStream,
                core::ptr::addr_of_mut!((*pData).dwMetaFileHeight) as *mut i8,
                core::mem::size_of::<DWORD>() as DWORD,
            );
            if dwSize != core::mem::size_of::<DWORD>() as DWORD {
                hRes = CONVERT10_E_OLESTREAM_PUT;
            }
        }

        if hRes == S_OK {
            dwSize = ((*(*pOleStream).lpstbl).Put.unwrap_unchecked())(
                pOleStream,
                core::ptr::addr_of_mut!((*pData).dwDataLength) as *mut i8,
                core::mem::size_of::<DWORD>() as DWORD,
            );
            if dwSize != core::mem::size_of::<DWORD>() as DWORD {
                hRes = CONVERT10_E_OLESTREAM_PUT;
            }
        }

        if hRes == S_OK {
            if (*pData).dwDataLength > 0 {
                dwSize = ((*(*pOleStream).lpstbl).Put.unwrap_unchecked())(
                    pOleStream,
                    (*pData).pData as *mut i8,
                    ((*pData).dwDataLength) as DWORD,
                );
                if dwSize != (*pData).dwDataLength {
                    hRes = CONVERT10_E_OLESTREAM_PUT;
                }
            }
        }
    }
    hRes
}

unsafe fn OLECONVERT_CreateOlePresStream(
    pStorage: *mut IStorage,
    dwExtentX: DWORD,
    dwExtentY: DWORD,
    pData: *mut BYTE,
    dwDataLength: DWORD,
) {
    let hRes: HRESULT;
    let mut pStream: *mut IStream = core::ptr::null_mut();
    static wstrStreamName: [u16; 12] = [2, b'O' as u16, b'l' as u16, b'e' as u16, b'P' as u16, b'r' as u16, b'e' as u16, b's' as u16, b'0' as u16, b'0' as u16, b'0' as u16, 0];
    static pOlePresStreamHeader: [BYTE; 28] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0x03, 0x00, 0x00, 0x00,
        0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00
    ];
    static pOlePresStreamHeaderEmpty: [BYTE; 24] = [
        0x00, 0x00, 0x00, 0x00,
        0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00
    ];

    

    hRes = IStorage_CreateStream(
        pStorage,
        wstrStreamName.as_ptr(),
        STGM_CREATE | STGM_WRITE | STGM_SHARE_EXCLUSIVE,
        0,
        0,
        &mut pStream,
    );

    if hRes == 0 {
        let nHeaderSize: DWORD;
        let mut OlePres: OLECONVERT_ISTORAGE_OLEPRES = core::mem::zeroed();

        memset(&mut OlePres as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of::<OLECONVERT_ISTORAGE_OLEPRES>());

        if dwDataLength > 0 {
            memcpy(
                OlePres.byUnknown1.as_mut_ptr() as *mut core::ffi::c_void,
                pOlePresStreamHeader.as_ptr() as *const core::ffi::c_void,
                core::mem::size_of_val(&pOlePresStreamHeader),
            );
            nHeaderSize = core::mem::size_of_val(&pOlePresStreamHeader) as DWORD;
        } else {
            memcpy(
                OlePres.byUnknown1.as_mut_ptr() as *mut core::ffi::c_void,
                pOlePresStreamHeaderEmpty.as_ptr() as *const core::ffi::c_void,
                core::mem::size_of_val(&pOlePresStreamHeaderEmpty),
            );
            nHeaderSize = core::mem::size_of_val(&pOlePresStreamHeaderEmpty) as DWORD;
        }

        OlePres.dwExtentX = dwExtentX;
        OlePres.dwExtentY = (dwExtentY as i32).wrapping_neg() as DWORD;

        if dwDataLength > core::mem::size_of::<METAFILEPICT16>() as DWORD {
            OlePres.dwSize = dwDataLength - core::mem::size_of::<METAFILEPICT16>() as DWORD;
            OlePres.pData = pData.add(8);
        }

        IStream_Write(pStream, OlePres.byUnknown1.as_ptr() as *const core::ffi::c_void, nHeaderSize, core::ptr::null_mut());
        IStream_Write(pStream, &OlePres.dwExtentX as *const _ as *const core::ffi::c_void, core::mem::size_of_val(&OlePres.dwExtentX) as ULONG, core::ptr::null_mut());
        IStream_Write(pStream, &OlePres.dwExtentY as *const _ as *const core::ffi::c_void, core::mem::size_of_val(&OlePres.dwExtentY) as ULONG, core::ptr::null_mut());
        IStream_Write(pStream, &OlePres.dwSize as *const _ as *const core::ffi::c_void, core::mem::size_of_val(&OlePres.dwSize) as ULONG, core::ptr::null_mut());
        if OlePres.dwSize > 0 {
            IStream_Write(pStream, OlePres.pData as *const core::ffi::c_void, OlePres.dwSize, core::ptr::null_mut());
        }
        IStream_Release(pStream);
    }
}

unsafe fn OLECONVERT_CreateOle10NativeStream(pStorage: *mut IStorage, pData: *const u8, dwDataLength: u32) {
    let hRes: HRESULT;
    let mut pStream: *mut IStream = core::ptr::null_mut();
    static wstrStreamName: [u16; 13] = [1, b'O' as u16, b'l' as u16, b'e' as u16, b'1' as u16, b'0' as u16, b'N' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'v' as u16, b'e' as u16, 0];

    hRes = IStorage_CreateStream(pStorage, wstrStreamName.as_ptr(),
        STGM_CREATE | STGM_WRITE | STGM_SHARE_EXCLUSIVE, 0, 0, &mut pStream);

    if hRes == S_OK {
        IStream_Write(pStream, &dwDataLength as *const _ as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
        IStream_Write(pStream, pData as *const core::ffi::c_void, dwDataLength, core::ptr::null_mut());
        IStream_Release(pStream);
    }
}

unsafe fn OLECONVERT_GetOLE10ProgID(pStorage: *mut IStorage, strProgID: *mut u8, dwSize: *mut u32) -> i32 {
    let mut hRes: i32;
    let mut pStream: *mut IStream = core::ptr::null_mut();
    let mut iSeekPos: LARGE_INTEGER = core::mem::zeroed();
    let mut CompObj: OLECONVERT_ISTORAGE_COMPOBJ = core::mem::zeroed();
    static wstrStreamName: [u16; 9] = [1, 'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'O' as u16, 'b' as u16, 'j' as u16, 0];

    hRes = IStorage_OpenStream(pStorage, wstrStreamName.as_ptr(), core::ptr::null_mut(),
        STGM_READ | STGM_SHARE_EXCLUSIVE, 0, &mut pStream);
    if hRes == 0 {
        iSeekPos.u.LowPart = (core::mem::size_of_val(&CompObj.byUnknown1)
            + core::mem::size_of_val(&CompObj.clsid)) as DWORD;
        iSeekPos.u.HighPart = 0;

        IStream_Seek(pStream, iSeekPos, 0, core::ptr::null_mut());
        IStream_Read(pStream, &mut CompObj.dwCLSIDNameLength as *mut u32 as *mut core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
        iSeekPos.u.LowPart = CompObj.dwCLSIDNameLength;
        IStream_Seek(pStream, iSeekPos, 1, core::ptr::null_mut());
        IStream_Read(pStream, &mut CompObj.dwOleTypeNameLength as *mut u32 as *mut core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
        iSeekPos.u.LowPart = CompObj.dwOleTypeNameLength;
        IStream_Seek(pStream, iSeekPos, 1, core::ptr::null_mut());

        IStream_Read(pStream, dwSize as *mut core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
        if *dwSize > 0 {
            IStream_Read(pStream, strProgID as *mut core::ffi::c_void, *dwSize, core::ptr::null_mut());
        }
        IStream_Release(pStream);
    } else {
        let mut stat: STATSTG = core::mem::zeroed();
        let mut wstrProgID: *mut u16 = core::ptr::null_mut();

        let clsid: *const GUID = &stat.clsid;
        IStorage_Stat(pStorage, &mut stat, STATFLAG_NONAME);
        hRes = ProgIDFromCLSID(clsid, &mut wstrProgID);
        if hRes == 0 {
            *dwSize = WideCharToMultiByte(
                CP_ACP,
                0,
                wstrProgID,
                -1,
                strProgID as *mut i8,
                *dwSize as c_int,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) as DWORD;
            CoTaskMemFree(wstrProgID as *mut core::ffi::c_void);
        }
    }
    hRes
}

unsafe fn OLECONVERT_GetOle10PresData(pStorage: *mut IStorage, pOleStreamData: *mut OLECONVERT_OLESTREAM_DATA)
{
    let hRes: HRESULT;
    let mut pStream: *mut IStream = core::ptr::null_mut();
    static wstrStreamName: [u16; 13] = [1, 'O' as u16, 'l' as u16, 'e' as u16, '1' as u16, '0' as u16, 'N' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 0];

    (*pOleStreamData.offset(0)).dwOleID = OLESTREAM_ID;
    (*pOleStreamData.offset(0)).dwTypeID = 2;
    (*pOleStreamData.offset(1)).dwOleID = OLESTREAM_ID;
    (*pOleStreamData.offset(1)).dwTypeID = 0;
    (*pOleStreamData.offset(0)).dwMetaFileWidth = 0;
    (*pOleStreamData.offset(0)).dwMetaFileHeight = 0;
    (*pOleStreamData.offset(0)).pData = core::ptr::null_mut();
    (*pOleStreamData.offset(1)).pData = core::ptr::null_mut();

    hRes = IStorage_OpenStream(pStorage, wstrStreamName.as_ptr(), core::ptr::null_mut(),
        STGM_READ | STGM_SHARE_EXCLUSIVE, 0, &mut pStream);
    if hRes == S_OK
    {
        IStream_Read(pStream, core::ptr::addr_of_mut!((*pOleStreamData).dwDataLength) as LPVOID, core::mem::size_of::<DWORD>() as u32, core::ptr::null_mut());
        if (*pOleStreamData).dwDataLength > 0
        {
            (*pOleStreamData).pData = HeapAlloc(GetProcessHeap(), 0, ((*pOleStreamData).dwDataLength as usize) as SIZE_T) as *mut BYTE;
            IStream_Read(pStream, (*pOleStreamData).pData as LPVOID, (*pOleStreamData).dwDataLength, core::ptr::null_mut());
        }
        IStream_Release(pStream);
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetConvertStg(stg: *mut IStorage) -> HRESULT {
    static stream_1oleW: [u16; 5] = [1, b'O' as u16, b'l' as u16, b'e' as u16, 0];
    const version_magic: u32 = 0x02000001;
    let mut header: [u32; 2] = [0; 2];
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut hr: HRESULT;

    

    TRACE!(b"%p\n\0".as_ptr() as *const c_char);

    if stg.is_null() {
        return 0x80070057u32 as HRESULT;
    }

    hr = IStorage_OpenStream(
        stg,
        stream_1oleW.as_ptr(),
        core::ptr::null_mut(),
        STGM_READ | STGM_SHARE_EXCLUSIVE,
        0,
        &mut stream,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = IStream_Read(
        stream,
        header.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of::<[u32; 2]>() as u32,
        core::ptr::null_mut(),
    );
    IStream_Release(stream);
    if FAILED(hr) {
        return hr;
    }

    if header[0] != version_magic {
        ERR(b"got wrong version magic for 1Ole stream, 0x%08x\n\0".as_ptr() as *const c_char);
        return 0x80004005u32 as HRESULT;
    }

    if header[1] & 0x00000004 != 0 {
        0u32 as HRESULT
    } else {
        1u32 as HRESULT
    }
}

// ===========================================================================
// File-scope vtbl tables.
//
// rustconv translates FUNCTIONS only, so every `static const ...Vtbl` in
// storage32.c was missing from the raw translation. These are transcribed
// slot-for-slot from the C (storage32.c:484, :1176, :5222, :5246, :5524, :5546,
// :6434, :6456, :6770, :6792) — same order, same functions, nothing omitted.
//
// The COM tables (IStorageVtbl / IEnumSTATSTGVtbl / IDirectWriterLockVtbl) come from
// com-sys, whose slots are `Option<unsafe extern "stdcall"/"C" fn(..)>` — hence the
// `Some(..)` and the `stdmethod!` ABI on every method above. StorageBaseImplVtbl is NOT
// a COM vtbl: it is a plain-C struct of plain fn pointers (cdecl on every arch,
// non-Option), so its slots take the function directly.
// ===========================================================================

/// storage32.c:484 `static const IDirectWriterLockVtbl DirectWriterLockVtbl`
pub static DirectWriterLockVtbl: IDirectWriterLockVtbl = IDirectWriterLockVtbl {
    QueryInterface: Some(directwriterlock_QueryInterface),
    AddRef: Some(directwriterlock_AddRef),
    Release: Some(directwriterlock_Release),
    WaitForWriteAccess: Some(directwriterlock_WaitForWriteAccess),
    ReleaseWriteAccess: Some(directwriterlock_ReleaseWriteAccess),
    HaveWriteAccess: Some(directwriterlock_HaveWriteAccess),
};

/// storage32.c:1176 `static const IEnumSTATSTGVtbl IEnumSTATSTGImpl_Vtbl`
pub static IEnumSTATSTGImpl_Vtbl: IEnumSTATSTGVtbl = IEnumSTATSTGVtbl {
    QueryInterface: Some(IEnumSTATSTGImpl_QueryInterface),
    AddRef: Some(IEnumSTATSTGImpl_AddRef),
    Release: Some(IEnumSTATSTGImpl_Release),
    Next: Some(IEnumSTATSTGImpl_Next),
    Skip: Some(IEnumSTATSTGImpl_Skip),
    Reset: Some(IEnumSTATSTGImpl_Reset),
    Clone: Some(IEnumSTATSTGImpl_Clone),
};

/// storage32.c:5222 `static const StorageBaseImplVtbl StorageImpl_BaseVtbl`
pub static StorageImpl_BaseVtbl: StorageBaseImplVtbl = StorageBaseImplVtbl {
    Destroy: StorageImpl_Destroy,
    Invalidate: StorageImpl_Invalidate,
    Flush: StorageImpl_Flush,
    GetFilename: StorageImpl_GetFilename,
    CreateDirEntry: StorageImpl_CreateDirEntry,
    WriteDirEntry: StorageImpl_BaseWriteDirEntry,
    ReadDirEntry: StorageImpl_BaseReadDirEntry,
    DestroyDirEntry: StorageImpl_DestroyDirEntry,
    StreamReadAt: StorageImpl_StreamReadAt,
    StreamWriteAt: StorageImpl_StreamWriteAt,
    StreamSetSize: StorageImpl_StreamSetSize,
    StreamLink: StorageImpl_StreamLink,
    GetTransactionSig: StorageImpl_GetTransactionSig,
    SetTransactionSig: StorageImpl_SetTransactionSig,
    LockTransaction: StorageImpl_LockTransaction,
    UnlockTransaction: StorageImpl_UnlockTransaction,
};

/// storage32.c:5246 `static const IStorageVtbl StorageImpl_Vtbl`
pub static StorageImpl_Vtbl: IStorageVtbl = IStorageVtbl {
    QueryInterface: Some(StorageBaseImpl_QueryInterface),
    AddRef: Some(StorageBaseImpl_AddRef),
    Release: Some(StorageBaseImpl_Release),
    CreateStream: Some(StorageBaseImpl_CreateStream),
    OpenStream: Some(StorageBaseImpl_OpenStream),
    CreateStorage: Some(StorageBaseImpl_CreateStorage),
    OpenStorage: Some(StorageBaseImpl_OpenStorage),
    CopyTo: Some(StorageBaseImpl_CopyTo),
    MoveElementTo: Some(StorageBaseImpl_MoveElementTo),
    Commit: Some(StorageBaseImpl_Commit),
    Revert: Some(StorageBaseImpl_Revert),
    EnumElements: Some(StorageBaseImpl_EnumElements),
    DestroyElement: Some(StorageBaseImpl_DestroyElement),
    RenameElement: Some(StorageBaseImpl_RenameElement),
    SetElementTimes: Some(StorageBaseImpl_SetElementTimes),
    SetClass: Some(StorageBaseImpl_SetClass),
    SetStateBits: Some(StorageBaseImpl_SetStateBits),
    Stat: Some(StorageBaseImpl_Stat),
};

/// storage32.c:5524 `static const IStorageVtbl StorageInternalImpl_Vtbl`
pub static StorageInternalImpl_Vtbl: IStorageVtbl = IStorageVtbl {
    QueryInterface: Some(StorageBaseImpl_QueryInterface),
    AddRef: Some(StorageBaseImpl_AddRef),
    Release: Some(StorageBaseImpl_Release),
    CreateStream: Some(StorageBaseImpl_CreateStream),
    OpenStream: Some(StorageBaseImpl_OpenStream),
    CreateStorage: Some(StorageBaseImpl_CreateStorage),
    OpenStorage: Some(StorageBaseImpl_OpenStorage),
    CopyTo: Some(StorageBaseImpl_CopyTo),
    MoveElementTo: Some(StorageBaseImpl_MoveElementTo),
    Commit: Some(StorageInternalImpl_Commit),
    Revert: Some(StorageInternalImpl_Revert),
    EnumElements: Some(StorageBaseImpl_EnumElements),
    DestroyElement: Some(StorageBaseImpl_DestroyElement),
    RenameElement: Some(StorageBaseImpl_RenameElement),
    SetElementTimes: Some(StorageBaseImpl_SetElementTimes),
    SetClass: Some(StorageBaseImpl_SetClass),
    SetStateBits: Some(StorageBaseImpl_SetStateBits),
    Stat: Some(StorageBaseImpl_Stat),
};

/// storage32.c:5546 `static const StorageBaseImplVtbl StorageInternalImpl_BaseVtbl`
pub static StorageInternalImpl_BaseVtbl: StorageBaseImplVtbl = StorageBaseImplVtbl {
    Destroy: StorageInternalImpl_Destroy,
    Invalidate: StorageInternalImpl_Invalidate,
    Flush: StorageInternalImpl_Flush,
    GetFilename: StorageInternalImpl_GetFilename,
    CreateDirEntry: StorageInternalImpl_CreateDirEntry,
    WriteDirEntry: StorageInternalImpl_WriteDirEntry,
    ReadDirEntry: StorageInternalImpl_ReadDirEntry,
    DestroyDirEntry: StorageInternalImpl_DestroyDirEntry,
    StreamReadAt: StorageInternalImpl_StreamReadAt,
    StreamWriteAt: StorageInternalImpl_StreamWriteAt,
    StreamSetSize: StorageInternalImpl_StreamSetSize,
    StreamLink: StorageInternalImpl_StreamLink,
    GetTransactionSig: StorageInternalImpl_GetTransactionSig,
    SetTransactionSig: StorageInternalImpl_SetTransactionSig,
    LockTransaction: StorageInternalImpl_LockTransaction,
    UnlockTransaction: StorageInternalImpl_UnlockTransaction,
};

/// storage32.c:6434 `static const IStorageVtbl TransactedSnapshotImpl_Vtbl`
pub static TransactedSnapshotImpl_Vtbl: IStorageVtbl = IStorageVtbl {
    QueryInterface: Some(StorageBaseImpl_QueryInterface),
    AddRef: Some(StorageBaseImpl_AddRef),
    Release: Some(StorageBaseImpl_Release),
    CreateStream: Some(StorageBaseImpl_CreateStream),
    OpenStream: Some(StorageBaseImpl_OpenStream),
    CreateStorage: Some(StorageBaseImpl_CreateStorage),
    OpenStorage: Some(StorageBaseImpl_OpenStorage),
    CopyTo: Some(StorageBaseImpl_CopyTo),
    MoveElementTo: Some(StorageBaseImpl_MoveElementTo),
    Commit: Some(TransactedSnapshotImpl_Commit),
    Revert: Some(TransactedSnapshotImpl_Revert),
    EnumElements: Some(StorageBaseImpl_EnumElements),
    DestroyElement: Some(StorageBaseImpl_DestroyElement),
    RenameElement: Some(StorageBaseImpl_RenameElement),
    SetElementTimes: Some(StorageBaseImpl_SetElementTimes),
    SetClass: Some(StorageBaseImpl_SetClass),
    SetStateBits: Some(StorageBaseImpl_SetStateBits),
    Stat: Some(StorageBaseImpl_Stat),
};

/// storage32.c:6456 `static const StorageBaseImplVtbl TransactedSnapshotImpl_BaseVtbl`
pub static TransactedSnapshotImpl_BaseVtbl: StorageBaseImplVtbl = StorageBaseImplVtbl {
    Destroy: TransactedSnapshotImpl_Destroy,
    Invalidate: TransactedSnapshotImpl_Invalidate,
    Flush: TransactedSnapshotImpl_Flush,
    GetFilename: TransactedSnapshotImpl_GetFilename,
    CreateDirEntry: TransactedSnapshotImpl_CreateDirEntry,
    WriteDirEntry: TransactedSnapshotImpl_WriteDirEntry,
    ReadDirEntry: TransactedSnapshotImpl_ReadDirEntry,
    DestroyDirEntry: TransactedSnapshotImpl_DestroyDirEntry,
    StreamReadAt: TransactedSnapshotImpl_StreamReadAt,
    StreamWriteAt: TransactedSnapshotImpl_StreamWriteAt,
    StreamSetSize: TransactedSnapshotImpl_StreamSetSize,
    StreamLink: TransactedSnapshotImpl_StreamLink,
    GetTransactionSig: TransactedSnapshotImpl_GetTransactionSig,
    SetTransactionSig: TransactedSnapshotImpl_SetTransactionSig,
    LockTransaction: TransactedSnapshotImpl_LockTransaction,
    UnlockTransaction: TransactedSnapshotImpl_UnlockTransaction,
};

/// storage32.c:6770 `static const IStorageVtbl TransactedSharedImpl_Vtbl`
pub static TransactedSharedImpl_Vtbl: IStorageVtbl = IStorageVtbl {
    QueryInterface: Some(StorageBaseImpl_QueryInterface),
    AddRef: Some(StorageBaseImpl_AddRef),
    Release: Some(StorageBaseImpl_Release),
    CreateStream: Some(StorageBaseImpl_CreateStream),
    OpenStream: Some(StorageBaseImpl_OpenStream),
    CreateStorage: Some(StorageBaseImpl_CreateStorage),
    OpenStorage: Some(StorageBaseImpl_OpenStorage),
    CopyTo: Some(StorageBaseImpl_CopyTo),
    MoveElementTo: Some(StorageBaseImpl_MoveElementTo),
    Commit: Some(TransactedSharedImpl_Commit),
    Revert: Some(TransactedSharedImpl_Revert),
    EnumElements: Some(StorageBaseImpl_EnumElements),
    DestroyElement: Some(StorageBaseImpl_DestroyElement),
    RenameElement: Some(StorageBaseImpl_RenameElement),
    SetElementTimes: Some(StorageBaseImpl_SetElementTimes),
    SetClass: Some(StorageBaseImpl_SetClass),
    SetStateBits: Some(StorageBaseImpl_SetStateBits),
    Stat: Some(StorageBaseImpl_Stat),
};

/// storage32.c:6792 `static const StorageBaseImplVtbl TransactedSharedImpl_BaseVtbl`
pub static TransactedSharedImpl_BaseVtbl: StorageBaseImplVtbl = StorageBaseImplVtbl {
    Destroy: TransactedSharedImpl_Destroy,
    Invalidate: TransactedSharedImpl_Invalidate,
    Flush: TransactedSharedImpl_Flush,
    GetFilename: TransactedSharedImpl_GetFilename,
    CreateDirEntry: TransactedSharedImpl_CreateDirEntry,
    WriteDirEntry: TransactedSharedImpl_WriteDirEntry,
    ReadDirEntry: TransactedSharedImpl_ReadDirEntry,
    DestroyDirEntry: TransactedSharedImpl_DestroyDirEntry,
    StreamReadAt: TransactedSharedImpl_StreamReadAt,
    StreamWriteAt: TransactedSharedImpl_StreamWriteAt,
    StreamSetSize: TransactedSharedImpl_StreamSetSize,
    StreamLink: TransactedSharedImpl_StreamLink,
    GetTransactionSig: TransactedSharedImpl_GetTransactionSig,
    SetTransactionSig: TransactedSharedImpl_SetTransactionSig,
    LockTransaction: TransactedSharedImpl_LockTransaction,
    UnlockTransaction: TransactedSharedImpl_UnlockTransaction,
};
