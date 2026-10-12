use crate::fltlib::fltlib::NtStatusToHResult;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_3CA582AA2320: [u16; 20] = [
    0x005Cu16, 0x0047, 0x006C, 0x006F, 0x0062, 0x0061, 0x006C, 0x003F, 0x003F, 0x005C,
    0x0046, 0x006C, 0x0074, 0x004D, 0x0067, 0x0072, 0x004D, 0x0073, 0x0067, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn FilterConnectCommunicationPort(
    lpPortName: LPCWSTR,
    dwOptions: DWORD,
    lpContext: LPCVOID,
    wSizeOfContext: WORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    hPort: *mut HANDLE,
) -> HRESULT {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut PortData: PFILTER_PORT_DATA = core::ptr::null_mut();
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut PortName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut PortNameSize: SIZE_T = 0;
    let mut BufferSize: SIZE_T = 0;
    let mut Ptr: PCHAR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut hr: HRESULT = 0;

    *hPort = INVALID_HANDLE_VALUE;

    /* Sanity check */
    if !lpContext.is_null() && wSizeOfContext == 0 {
        return E_INVALIDARG;
    }

    /* Get the length of the port name */
    PortNameSize = (wcslen(lpPortName) as SIZE_T)
        .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T);

    /* Calculate and allocate the size of the required buffer */
    BufferSize = (core::mem::size_of::<FILTER_PORT_DATA>() as SIZE_T)
        .wrapping_add(PortNameSize)
        .wrapping_add(wSizeOfContext as SIZE_T);
    PortData = RtlAllocateHeap(
        GetProcessHeap() as PVOID,
        0,
        BufferSize,
    ) as PFILTER_PORT_DATA;
    if PortData.is_null() {
        return E_OUTOFMEMORY;
    }

    /* Clear out the buffer and find the end of the fixed struct */
    RtlZeroMemory!(PortData as PVOID, BufferSize);
    Ptr = PortData.add(1 as usize) as PCHAR;

    (*PortData).Size = BufferSize as ULONG;
    (*PortData).Options = dwOptions as ULONG;

    /* Setup the port name */
    RtlInitUnicodeString(core::ptr::addr_of_mut!(PortName), lpPortName);
    (*PortData).PortName.Buffer = Ptr as PWSTR;
    (*PortData).PortName.MaximumLength = PortNameSize as USHORT;
    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!((*PortData).PortName),
        core::ptr::addr_of_mut!(PortName),
    );
    Ptr = Ptr.add((*PortData).PortName.Length as usize);

    /* Check if we were given a context */
    if !lpContext.is_null() {
        /* Add that into the buffer too */
        (*PortData).Context = Ptr as PVOID;
        RtlCopyMemory!(
            (*PortData).Context as PVOID,
            lpContext as *const core::ffi::c_void,
            wSizeOfContext as SIZE_T,
        );
    }

    /* Initialize the object attributes */
    RtlInitUnicodeString(core::ptr::addr_of_mut!(DeviceName), RUSTCONV_WSTR_3CA582AA2320.as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes =
        (OBJ_EXCLUSIVE | OBJ_CASE_INSENSITIVE) as ULONG;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(DeviceName) as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Check if we were passed any security attributes */
    if !lpSecurityAttributes.is_null() {
        /* Add these manually and update the flags if we were asked to make it inheritable */
        ObjectAttributes.SecurityDescriptor =
            (*lpSecurityAttributes).lpSecurityDescriptor as PVOID;
        if (*lpSecurityAttributes).bInheritHandle != 0 {
            ObjectAttributes.Attributes |= OBJ_INHERIT as ULONG;
        }
    }

    /* Now get a handle to the device */
    Status = NtCreateFile(
        core::ptr::addr_of_mut!(FileHandle),
        SYNCHRONIZE | FILE_READ_DATA | FILE_WRITE_DATA,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::null_mut(),
        0,
        0,
        FILE_OPEN_IF,
        0,
        PortData as PVOID,
        BufferSize as ULONG,
    );
    if NT_SUCCESS!(Status) {
        *hPort = FileHandle;
        hr = S_OK;
    } else {
        hr = NtStatusToHResult(Status);
    }

    /* Cleanup and return */
    RtlFreeHeap(GetProcessHeap() as PVOID, 0, PortData as PVOID);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn FilterSendMessage(
    hPort: HANDLE,
    lpInBuffer: LPVOID,
    dwInBufferSize: DWORD,
    lpOutBuffer: LPVOID,
    dwOutBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hPort);
    UNREFERENCED_PARAMETER!(lpInBuffer);
    UNREFERENCED_PARAMETER!(dwInBufferSize);
    UNREFERENCED_PARAMETER!(lpOutBuffer);
    UNREFERENCED_PARAMETER!(dwOutBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterGetMessage(
    hPort: HANDLE,
    lpMessageBuffer: PFILTER_MESSAGE_HEADER,
    dwMessageBufferSize: DWORD,
    lpOverlapped: LPOVERLAPPED,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hPort);
    UNREFERENCED_PARAMETER!(lpMessageBuffer);
    UNREFERENCED_PARAMETER!(dwMessageBufferSize);
    UNREFERENCED_PARAMETER!(lpOverlapped);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterReplyMessage(
    mut hPort: HANDLE,
    mut lpReplyBuffer: PFILTER_REPLY_HEADER,
    mut dwReplyBufferSize: DWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hPort);
    UNREFERENCED_PARAMETER!(lpReplyBuffer);
    UNREFERENCED_PARAMETER!(dwReplyBufferSize);
    return E_NOTIMPL;
}
