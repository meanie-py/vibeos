//! `storage32.h` — the compound-file (structured storage) shared surface.
//!
//! rustconv translates FUNCTIONS, not headers, so everything storage32.h declares —
//! the on-disk offsets, the `StorageBaseImpl` class hierarchy, its virtual-method
//! table, and the `static inline` dispatchers through it — is simply absent from the
//! translated `ring::storage32` / `ring::stg_stream` / `ring::stg_prop`. They *use*
//! these names (storage32.rs alone references DIRENTRY_NULL, BLOCK_*, OFFSET_* ~130
//! times) and declared them as `extern` symbols, which link to nothing.
//!
//! This module is that header, translated once, centrally. Everything here is
//! ole32-INTERNAL: with storage32.c/stg_stream.c/stg_prop.c/datacache.c all flipped to
//! Rust, no C TU sees these layouts any more, so the only correctness requirement is
//! self-consistency plus exact fidelity to the ON-DISK format (the OFFSET_*/BLOCK_*
//! constants and DirEntry, which really are wire format).
//!
//! ABI note on `StorageBaseImplVtbl`: these are PLAIN C function pointers
//! (`void (*Destroy)(StorageBaseImpl*)`), not COM methods — no STDMETHODCALLTYPE — so
//! they are cdecl on every arch, i.e. `extern "C"`, NOT `stdmethod!`. Getting this
//! backwards would break i386 only.
#![allow(clippy::all)]
use crate::prelude::*;
use crate::ring::storage32::BlockChainStream;

// ---------------------------------------------------------------------------
// On-disk format offsets (storage32.h). WIRE FORMAT — do not "tidy" these.
// ---------------------------------------------------------------------------
pub const OFFSET_MINORVERSION: ULONG = 0x00000018;
pub const OFFSET_MAJORVERSION: ULONG = 0x0000001a;
pub const OFFSET_BYTEORDERMARKER: ULONG = 0x0000001c;
pub const OFFSET_BIGBLOCKSIZEBITS: ULONG = 0x0000001e;
pub const OFFSET_SMALLBLOCKSIZEBITS: ULONG = 0x00000020;
pub const OFFSET_DIRSECTORCOUNT: ULONG = 0x00000028;
pub const OFFSET_BBDEPOTCOUNT: ULONG = 0x0000002C;
pub const OFFSET_ROOTSTARTBLOCK: ULONG = 0x00000030;
pub const OFFSET_TRANSACTIONSIG: ULONG = 0x00000034;
pub const OFFSET_SMALLBLOCKLIMIT: ULONG = 0x00000038;
pub const OFFSET_SBDEPOTSTART: ULONG = 0x0000003C;
pub const OFFSET_SBDEPOTCOUNT: ULONG = 0x00000040;
pub const OFFSET_EXTBBDEPOTSTART: ULONG = 0x00000044;
pub const OFFSET_EXTBBDEPOTCOUNT: ULONG = 0x00000048;
pub const OFFSET_BBDEPOTSTART: ULONG = 0x0000004C;
pub const OFFSET_PS_NAME: ULONG = 0x00000000;
pub const OFFSET_PS_NAMELENGTH: ULONG = 0x00000040;
pub const OFFSET_PS_STGTYPE: ULONG = 0x00000042;
pub const OFFSET_PS_LEFTCHILD: ULONG = 0x00000044;
pub const OFFSET_PS_RIGHTCHILD: ULONG = 0x00000048;
pub const OFFSET_PS_DIRROOT: ULONG = 0x0000004C;
pub const OFFSET_PS_GUID: ULONG = 0x00000050;
pub const OFFSET_PS_CTIMELOW: ULONG = 0x00000064;
pub const OFFSET_PS_CTIMEHIGH: ULONG = 0x00000068;
pub const OFFSET_PS_MTIMELOW: ULONG = 0x0000006C;
pub const OFFSET_PS_MTIMEHIGH: ULONG = 0x00000070;
pub const OFFSET_PS_STARTBLOCK: ULONG = 0x00000074;
pub const OFFSET_PS_SIZE: ULONG = 0x00000078;
pub const OFFSET_PS_SIZE_HIGH: ULONG = 0x0000007C;

pub const DEF_BIG_BLOCK_SIZE_BITS: WORD = 0x0009;
pub const MIN_BIG_BLOCK_SIZE_BITS: WORD = 0x0009;
pub const MAX_BIG_BLOCK_SIZE_BITS: WORD = 0x000c;
pub const DEF_SMALL_BLOCK_SIZE_BITS: WORD = 0x0006;
pub const DEF_BIG_BLOCK_SIZE: WORD = 0x0200;
pub const DEF_SMALL_BLOCK_SIZE: WORD = 0x0040;

pub const BLOCK_FIRST_SPECIAL: ULONG = 0xFFFFFFFB;
pub const BLOCK_EXTBBDEPOT: ULONG = 0xFFFFFFFC;
pub const BLOCK_SPECIAL: ULONG = 0xFFFFFFFD;
pub const BLOCK_END_OF_CHAIN: ULONG = 0xFFFFFFFE;
pub const BLOCK_UNUSED: ULONG = 0xFFFFFFFF;
pub const DIRENTRY_NULL: ULONG = 0xFFFFFFFF;

pub const DIRENTRY_NAME_MAX_LEN: usize = 0x20;
pub const DIRENTRY_NAME_BUFFER_LEN: usize = 0x40;
pub const RAW_DIRENTRY_SIZE: ULONG = 0x00000080;
pub const HEADER_SIZE: ULONG = 512;
pub const MIN_BIG_BLOCK_SIZE: ULONG = 0x200;
pub const MAX_BIG_BLOCK_SIZE: ULONG = 0x1000;

/// Type of child entry link.
pub const DIRENTRY_RELATION_PREVIOUS: ULONG = 0;
pub const DIRENTRY_RELATION_NEXT: ULONG = 1;
pub const DIRENTRY_RELATION_DIR: ULONG = 2;

/// Type constant used in files for the root storage. (STGTY_STORAGE/STREAM/... are the
/// PUBLIC ones and live in `consts`; this one is a storage32.h-private extension.)
pub const STGTY_ROOT: DWORD = 0x05;

pub const COUNT_BBDEPOTINHEADER: usize = 109;
/// FIXME (upstream): this value is stored in the header, but wine hard-codes it.
pub const LIMIT_TO_USE_SMALL_BLOCK: ULONG = 0x1000;
/// Number of BlockChainStream objects to cache in a StorageImpl.
pub const BLOCKCHAIN_CACHE_SIZE: usize = 4;

// ---------------------------------------------------------------------------
// Range locks. The storage format reserves 0x7fffff00-0x7fffffff for locking; because
// it reserves the whole containing block (min 512 bytes), 0x7ffffe00-0x7ffffeff is
// unusable too. Sub-ranges are wine's empirical guesses — keep them byte-exact.
// ---------------------------------------------------------------------------
pub const RANGELOCK_UNK1_FIRST: ULONG = 0x7ffffe00;
pub const RANGELOCK_UNK1_LAST: ULONG = 0x7fffff57;
pub const RANGELOCK_PRIORITY1_FIRST: ULONG = 0x7fffff58;
pub const RANGELOCK_PRIORITY1_LAST: ULONG = 0x7fffff6b;
pub const RANGELOCK_NOSNAPSHOT_FIRST: ULONG = 0x7fffff6c;
pub const RANGELOCK_NOSNAPSHOT_LAST: ULONG = 0x7fffff7f;
pub const RANGELOCK_COMMIT: ULONG = 0x7fffff80;
pub const RANGELOCK_PRIORITY2_FIRST: ULONG = 0x7fffff81;
pub const RANGELOCK_PRIORITY2_LAST: ULONG = 0x7fffff91;
pub const RANGELOCK_CHECKLOCKS: ULONG = 0x7fffff92;
pub const RANGELOCK_READ_FIRST: ULONG = 0x7fffff93;
pub const RANGELOCK_READ_LAST: ULONG = 0x7fffffa6;
pub const RANGELOCK_WRITE_FIRST: ULONG = 0x7fffffa7;
pub const RANGELOCK_WRITE_LAST: ULONG = 0x7fffffba;
pub const RANGELOCK_DENY_READ_FIRST: ULONG = 0x7fffffbb;
pub const RANGELOCK_DENY_READ_LAST: ULONG = 0x7fffffce;
pub const RANGELOCK_DENY_WRITE_FIRST: ULONG = 0x7fffffcf;
pub const RANGELOCK_DENY_WRITE_LAST: ULONG = 0x7fffffe2;
pub const RANGELOCK_UNK2_FIRST: ULONG = 0x7fffffe3;
pub const RANGELOCK_UNK2_LAST: ULONG = 0x7fffffff;
pub const RANGELOCK_TRANSACTION_FIRST: ULONG = RANGELOCK_COMMIT;
pub const RANGELOCK_TRANSACTION_LAST: ULONG = RANGELOCK_CHECKLOCKS;
pub const RANGELOCK_FIRST: ULONG = RANGELOCK_UNK1_FIRST;
pub const RANGELOCK_LAST: ULONG = RANGELOCK_UNK2_LAST;

// ---------------------------------------------------------------------------
// STGM field extractors (storage32.h macros).
// ---------------------------------------------------------------------------
#[inline]
pub fn STGM_ACCESS_MODE(stgm: DWORD) -> DWORD {
    stgm & 0x0000f
}
#[inline]
pub fn STGM_SHARE_MODE(stgm: DWORD) -> DWORD {
    stgm & 0x000f0
}
#[inline]
pub fn STGM_CREATE_MODE(stgm: DWORD) -> DWORD {
    stgm & 0x0f000
}

pub const STGM_KNOWN_FLAGS: DWORD = 0xf0ff
    | STGM_TRANSACTED
    | STGM_CONVERT
    | STGM_PRIORITY
    | STGM_NOSCRATCH
    | STGM_NOSNAPSHOT
    | STGM_DIRECT_SWMR
    | STGM_DELETEONRELEASE
    | STGM_SIMPLE;

// ---------------------------------------------------------------------------
// Endian conversion. All three of our targets are little-endian, so these are the
// identity — exactly the `#else` branch storage32.h takes when !WORDS_BIGENDIAN.
// The static assert is the tripwire if a big-endian target is ever added.
// ---------------------------------------------------------------------------
const _: () = assert!(cfg!(target_endian = "little"), "storage32 htole/lendian are identity only on LE");

#[inline]
pub fn htole16(x: WORD) -> WORD {
    x
}
#[inline]
pub fn htole32(x: DWORD) -> DWORD {
    x
}
#[inline]
pub fn lendian16toh(x: WORD) -> WORD {
    x
}
#[inline]
pub fn lendian32toh(x: DWORD) -> DWORD {
    x
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A reference to a directory entry in the file or a transacted cache.
pub type DirRef = ULONG;

/// Read/write view of a directory entry. ON-DISK-DERIVED — field order and widths
/// mirror the C exactly.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DirEntry {
    pub name: [WCHAR; DIRENTRY_NAME_MAX_LEN],
    pub sizeOfNameString: WORD,
    pub stgType: BYTE,
    pub leftChild: DirRef,
    pub rightChild: DirRef,
    pub dirRootEntry: DirRef,
    pub clsid: GUID,
    pub ctime: FILETIME,
    pub mtime: FILETIME,
    pub startingBlock: ULONG,
    pub size: ULARGE_INTEGER,
}

/// `enum swmr_mode`. Spelled as a C-enum-compatible int + bare consts rather than a
/// Rust enum, because the translations use the variant names unqualified
/// (`lockingrole = SWMR_Writer`) and store them in a struct field.
pub type swmr_mode = c_int;
pub const SWMR_None: swmr_mode = 0;
pub const SWMR_Writer: swmr_mode = 1;
pub const SWMR_Reader: swmr_mode = 2;

/// Base class of every IStorage implementation in the compound-file code.
#[repr(C)]
pub struct StorageBaseImpl {
    pub IStorage_iface: IStorage,
    /// interface for adding a properties stream
    pub IPropertySetStorage_iface: IPropertySetStorage,
    pub IDirectWriterLock_iface: IDirectWriterLock,
    /// C field name is `ref` — a Rust keyword. rustconv renames it; the translations
    /// spell it `ref_0`, so that is the name here.
    pub ref_: LONG,
    /// Stream tracking list.
    pub strmHead: list,
    /// Storage tracking list.
    pub storageHead: list,
    /// TRUE if this object has been invalidated.
    pub reverted: BOOL,
    /// Index of the directory entry of this storage.
    pub storageDirEntry: DirRef,
    pub baseVtbl: *const StorageBaseImplVtbl,
    /// Flags this storage was opened or created with.
    pub openFlags: DWORD,
    /// State bits appear to only be preserved while running, not in the stream.
    pub stateBits: DWORD,
    /// Was the storage created or opened? (STGM_SIMPLE's behaviour depends on it.)
    pub create: BOOL,
    /// If opened transacted, the object implementing the snapshot/cache.
    pub transactedChild: *mut StorageBaseImpl,
    pub lockingrole: swmr_mode,
}

/// Virtual methods for StorageBaseImpl objects. PLAIN C function pointers (cdecl on
/// every arch) — see the module header. Non-`Option` because the C declares them
/// non-nullable and every table below initializes all sixteen.
#[repr(C)]
pub struct StorageBaseImplVtbl {
    pub Destroy: unsafe extern "C" fn(*mut StorageBaseImpl),
    pub Invalidate: unsafe extern "C" fn(*mut StorageBaseImpl),
    pub Flush: unsafe extern "C" fn(*mut StorageBaseImpl) -> HRESULT,
    pub GetFilename: unsafe extern "C" fn(*mut StorageBaseImpl, *mut LPWSTR) -> HRESULT,
    pub CreateDirEntry:
        unsafe extern "C" fn(*mut StorageBaseImpl, *const DirEntry, *mut DirRef) -> HRESULT,
    pub WriteDirEntry:
        unsafe extern "C" fn(*mut StorageBaseImpl, DirRef, *const DirEntry) -> HRESULT,
    pub ReadDirEntry:
        unsafe extern "C" fn(*mut StorageBaseImpl, DirRef, *mut DirEntry) -> HRESULT,
    pub DestroyDirEntry: unsafe extern "C" fn(*mut StorageBaseImpl, DirRef) -> HRESULT,
    pub StreamReadAt: unsafe extern "C" fn(
        *mut StorageBaseImpl,
        DirRef,
        ULARGE_INTEGER,
        ULONG,
        *mut c_void,
        *mut ULONG,
    ) -> HRESULT,
    pub StreamWriteAt: unsafe extern "C" fn(
        *mut StorageBaseImpl,
        DirRef,
        ULARGE_INTEGER,
        ULONG,
        *const c_void,
        *mut ULONG,
    ) -> HRESULT,
    pub StreamSetSize:
        unsafe extern "C" fn(*mut StorageBaseImpl, DirRef, ULARGE_INTEGER) -> HRESULT,
    pub StreamLink: unsafe extern "C" fn(*mut StorageBaseImpl, DirRef, DirRef) -> HRESULT,
    pub GetTransactionSig:
        unsafe extern "C" fn(*mut StorageBaseImpl, *mut ULONG, BOOL) -> HRESULT,
    pub SetTransactionSig: unsafe extern "C" fn(*mut StorageBaseImpl, ULONG) -> HRESULT,
    pub LockTransaction: unsafe extern "C" fn(*mut StorageBaseImpl, BOOL) -> HRESULT,
    pub UnlockTransaction: unsafe extern "C" fn(*mut StorageBaseImpl, BOOL) -> HRESULT,
}
unsafe impl Sync for StorageBaseImplVtbl {}

/// The root storage — i.e. a document file.
#[repr(C)]
pub struct StorageImpl {
    pub base: StorageBaseImpl,

    // File header
    pub bigBlockSizeBits: WORD,
    pub smallBlockSizeBits: WORD,
    pub bigBlockSize: ULONG,
    pub smallBlockSize: ULONG,
    pub bigBlockDepotCount: ULONG,
    pub rootStartBlock: ULONG,
    pub smallBlockLimit: ULONG,
    pub smallBlockDepotStart: ULONG,
    pub extBigBlockDepotStart: ULONG,
    pub extBigBlockDepotLocations: *mut ULONG,
    pub extBigBlockDepotLocationsSize: ULONG,
    pub extBigBlockDepotCount: ULONG,
    pub bigBlockDepotStart: [ULONG; COUNT_BBDEPOTINHEADER],
    pub transactionSig: ULONG,

    pub extBlockDepotCached: [ULONG; (MAX_BIG_BLOCK_SIZE / 4) as usize],
    pub indexExtBlockDepotCached: ULONG,

    pub blockDepotCached: [ULONG; (MAX_BIG_BLOCK_SIZE / 4) as usize],
    pub indexBlockDepotCached: ULONG,
    pub prevFreeBlock: ULONG,

    /// All small blocks before this one are known to be in use.
    pub firstFreeSmallBlock: ULONG,

    // Abstraction of the big block chains for the chains of the header.
    pub rootBlockChain: *mut BlockChainStream,
    pub smallBlockDepotChain: *mut BlockChainStream,
    pub smallBlockRootChain: *mut BlockChainStream,

    /// Cache of block chain stream objects for directory entries.
    pub blockChainCache: [*mut BlockChainStream; BLOCKCHAIN_CACHE_SIZE],
    pub blockChainToEvict: UINT,

    pub locks_supported: ULONG,
    pub lockBytes: *mut ILockBytes,
    pub locked_bytes: [ULONG; 8],
}

/// IStream over a stream inside a storage object.
#[repr(C)]
pub struct StgStreamImpl {
    pub IStream_iface: IStream,
    /// C field name is `ref` — see StorageBaseImpl::ref_0.
    pub ref_: LONG,
    /// We are an entry in the storage object's stream handler list.
    pub StrmListEntry: list,
    /// Storage that owns this stream.
    pub parentStorage: *mut StorageBaseImpl,
    /// Access mode of this stream.
    pub grfMode: DWORD,
    /// Index of the directory entry that owns (points to) this stream.
    pub dirEntry: DirRef,
    /// Current position of the cursor in the stream.
    pub currentPosition: ULARGE_INTEGER,
}

#[inline]
pub unsafe fn impl_from_IStream(iface: *mut IStream) -> *mut StgStreamImpl {
    CONTAINING_RECORD!(iface, StgStreamImpl, IStream_iface)
}

// ---------------------------------------------------------------------------
// The `static inline` dispatchers through baseVtbl.
// ---------------------------------------------------------------------------
#[inline]
pub unsafe fn StorageBaseImpl_Destroy(This: *mut StorageBaseImpl) {
    ((*(*This).baseVtbl).Destroy)(This)
}
#[inline]
pub unsafe fn StorageBaseImpl_Invalidate(This: *mut StorageBaseImpl) {
    ((*(*This).baseVtbl).Invalidate)(This)
}
#[inline]
pub unsafe fn StorageBaseImpl_Flush(This: *mut StorageBaseImpl) -> HRESULT {
    ((*(*This).baseVtbl).Flush)(This)
}
#[inline]
pub unsafe fn StorageBaseImpl_GetFilename(
    This: *mut StorageBaseImpl,
    result: *mut LPWSTR,
) -> HRESULT {
    ((*(*This).baseVtbl).GetFilename)(This, result)
}
#[inline]
pub unsafe fn StorageBaseImpl_CreateDirEntry(
    This: *mut StorageBaseImpl,
    newData: *const DirEntry,
    index: *mut DirRef,
) -> HRESULT {
    ((*(*This).baseVtbl).CreateDirEntry)(This, newData, index)
}
#[inline]
pub unsafe fn StorageBaseImpl_WriteDirEntry(
    This: *mut StorageBaseImpl,
    index: DirRef,
    data: *const DirEntry,
) -> HRESULT {
    ((*(*This).baseVtbl).WriteDirEntry)(This, index, data)
}
#[inline]
pub unsafe fn StorageBaseImpl_ReadDirEntry(
    This: *mut StorageBaseImpl,
    index: DirRef,
    data: *mut DirEntry,
) -> HRESULT {
    ((*(*This).baseVtbl).ReadDirEntry)(This, index, data)
}
#[inline]
pub unsafe fn StorageBaseImpl_DestroyDirEntry(
    This: *mut StorageBaseImpl,
    index: DirRef,
) -> HRESULT {
    ((*(*This).baseVtbl).DestroyDirEntry)(This, index)
}
/// Read up to `size` bytes from this directory entry's stream at the given offset.
#[inline]
pub unsafe fn StorageBaseImpl_StreamReadAt(
    This: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *mut c_void,
    bytesRead: *mut ULONG,
) -> HRESULT {
    ((*(*This).baseVtbl).StreamReadAt)(This, index, offset, size, buffer, bytesRead)
}
/// Write `size` bytes to this directory entry's stream at the given offset, growing
/// the stream if necessary.
#[inline]
pub unsafe fn StorageBaseImpl_StreamWriteAt(
    This: *mut StorageBaseImpl,
    index: DirRef,
    offset: ULARGE_INTEGER,
    size: ULONG,
    buffer: *const c_void,
    bytesWritten: *mut ULONG,
) -> HRESULT {
    ((*(*This).baseVtbl).StreamWriteAt)(This, index, offset, size, buffer, bytesWritten)
}
#[inline]
pub unsafe fn StorageBaseImpl_StreamSetSize(
    This: *mut StorageBaseImpl,
    index: DirRef,
    newsize: ULARGE_INTEGER,
) -> HRESULT {
    ((*(*This).baseVtbl).StreamSetSize)(This, index, newsize)
}
/// Make `dst` point to the same stream `src` points to. Other stream operations do not
/// work properly while two entries alias one stream, so this must be a very temporary
/// state and only one of them may be reachable at a time.
#[inline]
pub unsafe fn StorageBaseImpl_StreamLink(
    This: *mut StorageBaseImpl,
    dst: DirRef,
    src: DirRef,
) -> HRESULT {
    ((*(*This).baseVtbl).StreamLink)(This, dst, src)
}
#[inline]
pub unsafe fn StorageBaseImpl_GetTransactionSig(
    This: *mut StorageBaseImpl,
    result: *mut ULONG,
    refresh: BOOL,
) -> HRESULT {
    ((*(*This).baseVtbl).GetTransactionSig)(This, result, refresh)
}
#[inline]
pub unsafe fn StorageBaseImpl_SetTransactionSig(
    This: *mut StorageBaseImpl,
    value: ULONG,
) -> HRESULT {
    ((*(*This).baseVtbl).SetTransactionSig)(This, value)
}
#[inline]
pub unsafe fn StorageBaseImpl_LockTransaction(
    This: *mut StorageBaseImpl,
    write: BOOL,
) -> HRESULT {
    ((*(*This).baseVtbl).LockTransaction)(This, write)
}
#[inline]
pub unsafe fn StorageBaseImpl_UnlockTransaction(
    This: *mut StorageBaseImpl,
    write: BOOL,
) -> HRESULT {
    ((*(*This).baseVtbl).UnlockTransaction)(This, write)
}

// ---------------------------------------------------------------------------
// stg_prop.c / propidl.h helpers that are macros in C.
// ---------------------------------------------------------------------------
/// propidl.idl: `#define PROPSETHDR_OSVER_KIND(dwOSVer) HIWORD((dwOSVer))`
#[inline]
pub fn PROPSETHDR_OSVER_KIND(dwOSVer: DWORD) -> DWORD {
    HIWORD(dwOSVer) as DWORD
}

/// propidl.h: `PropVariantInit(pvar)` zeroes the whole PROPVARIANT.
#[inline]
pub unsafe fn PropVariantInit(pvar: *mut PROPVARIANT) {
    core::ptr::write_bytes(pvar as *mut u8, 0, core::mem::size_of::<PROPVARIANT>());
}
