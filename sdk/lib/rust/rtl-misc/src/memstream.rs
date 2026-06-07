use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{HANDLE, HRESULT, LARGE_INTEGER, NtReadVirtualMemory, PULARGE_INTEGER, RtlNtStatusToDosError, ULARGE_INTEGER};

// ---- local types (not in ntkrnl_sys) ----
type REFIID = *const GUID;
type VOID = core::ffi::c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

#[repr(C)]
pub struct _RTL_MEMORY_STREAM {
    pub Vtbl: *mut IStreamVtbl,
    pub RefCount: LONG,
    pub Unk1: ULONG,
    pub Current: PVOID,
    pub Start: PVOID,
    pub End: PVOID,
    pub FinalRelease: Option<unsafe extern "system" fn(Stream: PRTL_MEMORY_STREAM)>,
    pub ProcessHandle: HANDLE,
}
pub type RTL_MEMORY_STREAM = _RTL_MEMORY_STREAM;
pub type PRTL_MEMORY_STREAM = *mut _RTL_MEMORY_STREAM;

#[repr(C)]
pub struct IStream {
    pub lpVtbl: *const IStreamVtbl,
}
// Opaque for correctness; the vtable is what matters.
pub type IStreamVtbl = _IStreamVtbl;

#[repr(C)]
pub struct _IStreamVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IStream, riid: REFIID, ppvObject: *mut PVOID) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Read: unsafe extern "system" fn(This: *mut IStream, pv: PVOID, cb: ULONG, pcbRead: PULONG) -> HRESULT,
    pub Write: unsafe extern "system" fn(This: *mut IStream, pv: *const core::ffi::c_void, cb: ULONG, pcbWritten: PULONG) -> HRESULT,
    pub Seek: unsafe extern "system" fn(This: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: ULONG, plibNewPosition: PULARGE_INTEGER) -> HRESULT,
    pub SetSize: unsafe extern "system" fn(This: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT,
    pub CopyTo: unsafe extern "system" fn(This: *mut IStream, pstm: *mut IStream, cb: ULARGE_INTEGER, pcbRead: PULARGE_INTEGER, pcbWritten: PULARGE_INTEGER) -> HRESULT,
    pub Commit: unsafe extern "system" fn(This: *mut IStream, grfCommitFlags: ULONG) -> HRESULT,
    pub Revert: unsafe extern "system" fn(This: *mut IStream) -> HRESULT,
    pub LockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: ULONG) -> HRESULT,
    pub UnlockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: ULONG) -> HRESULT,
    pub Stat: unsafe extern "system" fn(This: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: ULONG) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT,
}

#[repr(C)]
pub struct FILETIME { pub dwLowDateTime: u32, pub dwHighDateTime: u32 }

#[repr(C)]
pub struct STATSTG {
    pub pwcsName: *mut u16,
    pub type_: ULONG,
    pub cbSize: ULARGE_INTEGER,
    pub mtime: FILETIME,
    pub ctime: FILETIME,
    pub atime: FILETIME,
    pub grfMode: ULONG,
    pub grfLocksSupported: ULONG,
    pub clsid: GUID,
    pub grfStateBits: ULONG,
    pub reserved: ULONG,
}

// ---- constants ----
const S_OK: HRESULT = 0;
const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT;
const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT;
const STG_E_INVALIDPOINTER: HRESULT = 0x80030009u32 as HRESULT;
const STGTY_STREAM: ULONG = 2;

// IIDs (must be static to take address)
static IID_IUnknown: GUID = GUID { Data1: 0x00000000, Data2: 0x0000, Data3: 0x0000, Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };
static IID_ISequentialStream: GUID = GUID { Data1: 0x0c733a30, Data2: 0x2a1c, Data3: 0x11ce, Data4: [0xad, 0xe5, 0x00, 0xaa, 0x00, 0x44, 0x77, 0x3d] };
static IID_IStream: GUID = GUID { Data1: 0x0000000c, Data2: 0x0000, Data3: 0x0000, Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };

// IsEqualGUID is a C macro, not an export — use inline compare

// COM helpers
#[inline]
fn FAILED(hr: HRESULT) -> bool {
    hr < 0
}

#[inline(always)]
unsafe fn is_equal_guid(a: *const GUID, b: *const GUID) -> bool {
    *(a as *const [u8; 16]) == *(b as *const [u8; 16])
}

#[inline(always)]
fn hresult_from_win32(x: ULONG) -> HRESULT {
    if (x as HRESULT) <= 0 { x as HRESULT } else { ((x & 0xFFFF) | (7 << 16) | 0x8000_0000) as HRESULT }
}

// IStream_* are C vtable-dispatch macros — call through lpVtbl directly
#[inline(always)]
unsafe fn IStream_AddRef(pstm: *mut IStream) -> ULONG {
    ((*(*pstm).lpVtbl).AddRef)(pstm)
}
#[inline(always)]
unsafe fn IStream_Read(pstm: *mut IStream, pv: PVOID, cb: ULONG, pcbRead: PULONG) -> HRESULT {
    ((*(*pstm).lpVtbl).Read)(pstm, pv, cb, pcbRead)
}
#[inline(always)]
unsafe fn IStream_Write(pstm: *mut IStream, pv: *const core::ffi::c_void, cb: ULONG, pcbWritten: PULONG) -> HRESULT {
    ((*(*pstm).lpVtbl).Write)(pstm, pv, cb, pcbWritten)
}

// ---- file-scope preamble (translated) ----
pub static RtlMemoryStreamVtbl: IStreamVtbl = IStreamVtbl {
    QueryInterface: RtlQueryInterfaceMemoryStream,
    AddRef: RtlAddRefMemoryStream,
    Release: RtlReleaseMemoryStream,
    Read: RtlReadMemoryStream,
    Write: RtlWriteMemoryStream,
    Seek: RtlSeekMemoryStream,
    SetSize: RtlSetMemoryStreamSize,
    CopyTo: RtlCopyMemoryStreamTo,
    Commit: RtlCommitMemoryStream,
    Revert: RtlRevertMemoryStream,
    LockRegion: RtlLockMemoryStreamRegion,
    UnlockRegion: RtlUnlockMemoryStreamRegion,
    Stat: RtlStatMemoryStream,
    Clone: RtlCloneMemoryStream,
};

pub static RtlOutOfProcessMemoryStreamVtbl: IStreamVtbl = IStreamVtbl {
    QueryInterface: RtlQueryInterfaceMemoryStream,
    AddRef: RtlAddRefMemoryStream,
    Release: RtlReleaseMemoryStream,
    Read: RtlReadOutOfProcessMemoryStream,
    Write: RtlWriteMemoryStream,
    Seek: RtlSeekMemoryStream,
    SetSize: RtlSetMemoryStreamSize,
    CopyTo: RtlCopyMemoryStreamTo,
    Commit: RtlCommitMemoryStream,
    Revert: RtlRevertMemoryStream,
    LockRegion: RtlLockMemoryStreamRegion,
    UnlockRegion: RtlUnlockMemoryStreamRegion,
    Stat: RtlStatMemoryStream,
    Clone: RtlCloneMemoryStream,
};

unsafe fn IStream_To_RTL_MEMORY_STREAM(
    Interface: *mut IStream,
) -> PRTL_MEMORY_STREAM {
    if Interface.is_null() {
        return core::ptr::null_mut();
    }

    (Interface as *mut u8).offset(-(core::mem::offset_of!(RTL_MEMORY_STREAM, Vtbl) as isize)) as PRTL_MEMORY_STREAM
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitMemoryStream(Stream: PRTL_MEMORY_STREAM) {
    core::ptr::write_bytes(Stream as *mut u8, 0, core::mem::size_of::<RTL_MEMORY_STREAM>());
    (*Stream).Vtbl = &RtlMemoryStreamVtbl as *const _ as *mut _;
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitOutOfProcessMemoryStream(
    Stream: PRTL_MEMORY_STREAM,
) {
    core::ptr::write_bytes(Stream as *mut u8, 0, core::mem::size_of::<RTL_MEMORY_STREAM>());
    (*Stream).Vtbl = &RtlOutOfProcessMemoryStreamVtbl as *const _ as *mut _;
    (*Stream).FinalRelease = Some(RtlFinalReleaseOutOfProcessMemoryStream);
}

#[no_mangle]
pub unsafe extern "system" fn RtlFinalReleaseOutOfProcessMemoryStream(
    Stream: PRTL_MEMORY_STREAM,
) {
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryInterfaceMemoryStream(
    This: *mut IStream,
    RequestedIid: REFIID,
    ResultObject: *mut PVOID,
) -> HRESULT {
    if is_equal_guid(RequestedIid, &raw const IID_IUnknown)
        || is_equal_guid(RequestedIid, &raw const IID_ISequentialStream)
        || is_equal_guid(RequestedIid, &raw const IID_IStream)
    {
        IStream_AddRef(This);
        core::ptr::write_volatile(ResultObject, This as PVOID);
        return S_OK;
    }

    core::ptr::write_volatile(ResultObject, core::ptr::null_mut());
    E_NOINTERFACE
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddRefMemoryStream(
    This: *mut IStream,
) -> ULONG {
    let Stream: PRTL_MEMORY_STREAM = IStream_To_RTL_MEMORY_STREAM(This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*Stream).RefCount)) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn RtlReleaseMemoryStream(
    This: *mut IStream,
) -> ULONG {
    let Stream: PRTL_MEMORY_STREAM = IStream_To_RTL_MEMORY_STREAM(This);
    let Result: LONG;

    Result = InterlockedDecrement(core::ptr::addr_of_mut!((*Stream).RefCount));

    if Result == 0 {
        if let Some(FinalRelease) = (*Stream).FinalRelease {
            FinalRelease(Stream);
        }
    }

    Result as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn RtlReadMemoryStream(
    This: *mut IStream,
    Buffer: PVOID,
    Length: ULONG,
    BytesRead: PULONG,
) -> HRESULT {
    let Stream: PRTL_MEMORY_STREAM = IStream_To_RTL_MEMORY_STREAM(This);
    let Available: SIZE_T =
        ((*Stream).End as PUCHAR as usize - (*Stream).Current as PUCHAR as usize) as SIZE_T;

    if !BytesRead.is_null() {
        core::ptr::write_volatile(BytesRead, 0);
    }

    if Length == 0 {
        return S_OK;
    }

    let CopyLength: ULONG = if (Available as ULONG) < Length {
        Available as ULONG
    } else {
        Length
    };

    core::ptr::copy(
        (*Stream).Current as *const u8,
        Buffer as *mut u8,
        CopyLength as usize,
    );

    (*Stream).Current = ((*Stream).Current as PUCHAR).add(CopyLength as usize) as PVOID;

    core::ptr::write_volatile(BytesRead, CopyLength);

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RtlReadOutOfProcessMemoryStream(
    This: *mut IStream,
    Buffer: PVOID,
    Length: ULONG,
    BytesRead: PULONG,
) -> HRESULT {
    let Status: NTSTATUS;
    let CopyLength: ULONG;
    let Stream: PRTL_MEMORY_STREAM = IStream_To_RTL_MEMORY_STREAM(This);
    let Available: SIZE_T =
        ((*Stream).End as PUCHAR as usize - (*Stream).Current as PUCHAR as usize) as SIZE_T;
    let mut LocalBytesRead: SIZE_T = 0;

    if !BytesRead.is_null() {
        core::ptr::write_volatile(BytesRead, 0);
    }

    if Length == 0 {
        return S_OK;
    }

    CopyLength = core::cmp::min(Available as ULONG, Length);

    Status = NtReadVirtualMemory(
        (*Stream).ProcessHandle,
        (*Stream).Current,
        Buffer,
        (CopyLength as SIZE_T).try_into().unwrap(),
        &mut LocalBytesRead as *mut SIZE_T as *mut _,
    );

    if NT_SUCCESS(Status) {
        (*Stream).Current = ((*Stream).Current as PUCHAR).add(LocalBytesRead as usize) as PVOID;
        if !BytesRead.is_null() {
            core::ptr::write_volatile(BytesRead, LocalBytesRead as ULONG);
        }
    }

    hresult_from_win32(RtlNtStatusToDosError(Status))
}

#[no_mangle]
pub unsafe extern "system" fn RtlSeekMemoryStream(
    This: *mut IStream,
    RelativeOffset: LARGE_INTEGER,
    Origin: ULONG,
    ResultOffset: PULARGE_INTEGER,
) -> HRESULT {
    let NewPosition: PVOID;
    let Stream: PRTL_MEMORY_STREAM = IStream_To_RTL_MEMORY_STREAM(This);

    const STREAM_SEEK_SET: ULONG = 0;
    const STREAM_SEEK_CUR: ULONG = 1;
    const STREAM_SEEK_END: ULONG = 2;

    match Origin {
        STREAM_SEEK_SET => {
            NewPosition = ((*Stream).Start as *mut UCHAR)
                .offset(RelativeOffset.QuadPart as isize) as PVOID;
        }
        STREAM_SEEK_CUR => {
            NewPosition = ((*Stream).Current as *mut UCHAR)
                .offset(RelativeOffset.QuadPart as isize) as PVOID;
        }
        STREAM_SEEK_END => {
            NewPosition = ((*Stream).End as *mut UCHAR)
                .offset(-(RelativeOffset.QuadPart as isize)) as PVOID;
        }
        _ => {
            return E_INVALIDARG;
        }
    }

    if NewPosition < (*Stream).Start || NewPosition > (*Stream).End {
        return STG_E_INVALIDPOINTER;
    }

    (*Stream).Current = NewPosition;

    if !ResultOffset.is_null() {
        (*ResultOffset).QuadPart =
            ((*Stream).Current as *mut UCHAR).offset_from((*Stream).Start as *mut UCHAR) as u64;
    }

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RtlCopyMemoryStreamTo(
    This: *mut IStream,
    Target: *mut IStream,
    Length: ULARGE_INTEGER,
    BytesRead: PULARGE_INTEGER,
    BytesWritten: PULARGE_INTEGER,
) -> HRESULT {
    let mut Buffer: [CHAR; 1024] = [0; 1024];
    let mut TotalSize: ULONGLONG;
    let mut Left: ULONG;
    let mut Amount: ULONG = 0;
    let mut Result: HRESULT = S_OK as HRESULT;

    if !BytesRead.is_null() {
        (*BytesRead).QuadPart = 0;
    }
    if !BytesWritten.is_null() {
        (*BytesWritten).QuadPart = 0;
    }

    if Target.is_null() {
        return S_OK as HRESULT;
    }

    if Length.QuadPart == 0 {
        return S_OK as HRESULT;
    }

    TotalSize = Length.QuadPart;
    while TotalSize != 0 {
        Left = core::cmp::min(TotalSize, core::mem::size_of::<[CHAR; 1024]>() as ULONGLONG) as ULONG;

        Result = IStream_Read(
            This,
            Buffer.as_mut_ptr() as *mut core::ffi::c_void,
            Left,
            &mut Amount,
        );
        if !BytesRead.is_null() {
            (*BytesRead).QuadPart += Amount as ULONGLONG;
        }
        if FAILED(Result) || Amount == 0 {
            break;
        }

        Left = Amount;

        Result = IStream_Write(
            Target,
            Buffer.as_mut_ptr() as *const core::ffi::c_void,
            Left,
            &mut Amount,
        );
        if !BytesWritten.is_null() {
            (*BytesWritten).QuadPart += Amount as ULONGLONG;
        }
        if FAILED(Result) || Amount != Left {
            break;
        }

        TotalSize -= Left as ULONGLONG;
    }
    Result
}

#[no_mangle]
pub unsafe extern "system" fn RtlStatMemoryStream(
    This: *mut IStream,
    Stats: *mut STATSTG,
    Flags: ULONG,
) -> HRESULT {
    let Stream: PRTL_MEMORY_STREAM = IStream_To_RTL_MEMORY_STREAM(This);

    if Stats.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    RtlZeroMemory(Stats as *mut _, core::mem::size_of::<STATSTG>() as _);
    (*Stats).type_ = STGTY_STREAM as ULONG;
    (*Stats).cbSize.QuadPart =
        ((*Stream).End as PUCHAR as isize - (*Stream).Start as PUCHAR as isize) as u64;

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RtlWriteMemoryStream(
    This: *mut IStream,
    Buffer: *const ::core::ffi::c_void,
    Length: ULONG,
    BytesWritten: PULONG,
) -> HRESULT {
    let _ = This;
    let _ = Buffer;
    let _ = Length;
    let _ = BytesWritten;

    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetMemoryStreamSize(
    This: *mut IStream,
    NewSize: ULARGE_INTEGER,
) -> HRESULT {
    let _ = This;
    let _ = NewSize;

    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn RtlCommitMemoryStream(
    This: *mut IStream,
    CommitFlags: ULONG,
) -> HRESULT {
    let _ = This;
    let _ = CommitFlags;

    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn RtlRevertMemoryStream(This: *mut IStream) -> HRESULT {
    let _ = This;
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn RtlLockMemoryStreamRegion(
    This: *mut IStream,
    Offset: ULARGE_INTEGER,
    Length: ULARGE_INTEGER,
    LockType: ULONG,
) -> HRESULT {
    let _ = This;
    let _ = Offset;
    let _ = Length;
    let _ = LockType;

    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnlockMemoryStreamRegion(
    This: *mut IStream,
    Offset: ULARGE_INTEGER,
    Length: ULARGE_INTEGER,
    LockType: ULONG,
) -> HRESULT {
    let _ = This;
    let _ = Offset;
    let _ = Length;
    let _ = LockType;

    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn RtlCloneMemoryStream(
    This: *mut IStream,
    ResultStream: *mut *mut IStream,
) -> HRESULT {
    let _ = This;
    let _ = ResultStream;

    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn RtlCopyMappedMemory(
    Destination: PVOID,
    Source: *const VOID,
    Size: SIZE_T,
) -> NTSTATUS {
    let Status: NTSTATUS = STATUS_SUCCESS;
    core::ptr::copy_nonoverlapping(Source as *const u8, Destination as *mut u8, Size as usize);
    Status
}
