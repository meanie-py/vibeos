use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_9112492AFDA5: [u16; 12] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0054, 0x0063,
    0x0070, 0x0000,
];

#[no_mangle]
pub static mut TcpFileName: PWCHAR = unsafe {
    RUSTCONV_C2R_WSTR_9112492AFDA5.as_ptr() as PWCHAR
};
#[no_mangle]
pub unsafe extern "C" fn openTcpFile(tcpFile: PHANDLE, DesiredAccess: ACCESS_MASK) -> NTSTATUS {
    let mut fileName: _UNICODE_STRING = core::mem::zeroed();
    let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ioStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let status: NTSTATUS;

    RtlInitUnicodeString(core::ptr::addr_of_mut!(fileName), TcpFileName as PCWSTR);

    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    objectAttributes.RootDirectory = core::ptr::null_mut();
    objectAttributes.ObjectName = core::ptr::addr_of_mut!(fileName);
    objectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    objectAttributes.SecurityDescriptor = core::ptr::null_mut();
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    status = NtOpenFile(
        tcpFile,
        DesiredAccess | SYNCHRONIZE as ACCESS_MASK,
        core::ptr::addr_of_mut!(objectAttributes),
        core::ptr::addr_of_mut!(ioStatusBlock),
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
        FILE_SYNCHRONOUS_IO_NONALERT as ULONG,
    );

    /* String does not need to be freed: it points to the constant
     * string we provided */

    if !NT_SUCCESS!(status) {
        *tcpFile = INVALID_HANDLE_VALUE as HANDLE;
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn closeTcpFile( h: HANDLE ) {
    ASSERT!(h != INVALID_HANDLE_VALUE);

    NtClose( h );
}
