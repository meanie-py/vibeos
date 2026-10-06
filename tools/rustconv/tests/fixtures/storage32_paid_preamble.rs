pub static STORAGE_magic: [u8; 8] = [0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];
pub static STORAGE_oldmagic: [u8; 8] = [0xd0, 0xcf, 0x11, 0xe0, 0x0e, 0x11, 0xfc, 0x0d];

#[repr(C)]
pub struct StorageInternalImpl {
    pub base: StorageBaseImpl,
    pub ParentListEntry: LIST_ENTRY,
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
    pub data: [u8; MAX_BIG_BLOCK_SIZE],
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