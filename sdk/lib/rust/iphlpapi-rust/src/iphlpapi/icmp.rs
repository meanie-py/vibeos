use crate::prelude::*;
use core::ffi::c_void;

// file-scope preamble HAND-WRITTEN: DeepSeek returned empty content for this unit
// (finish_reason=stop, ~1K output tokens — reasoning emitted, no code). Not a budget
// problem (the 64K cap was nowhere near hit); the model simply declines a nearly-empty
// file scope. Same mode as the dnsapi/dhcpcsvc/netapi32 preambles, all hand-assembled.
// The C file scope here is just includes + WINE_DEFAULT_DEBUG_CHANNEL, so there is
// nothing to carry over beyond the debug channel the prelude already supplies.


// ---- IcmpParseReplies (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn IcmpParseReplies(
    ReplyBuffer: LPVOID,
    ReplySize: u32,
) -> u32 {
    if ReplyBuffer.is_null() || ReplySize == 0 {
        return 0;
    }
    let pEcho: PICMP_ECHO_REPLY = ReplyBuffer as PICMP_ECHO_REPLY;
    if (*pEcho).Reserved == 0 {
        SetLastError((*pEcho).Status);
    }
    let nReplies = (*pEcho).Reserved;
    (*pEcho).Reserved = 0;
    (nReplies as u32)
}

// ---- Icmp6CreateFile (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn Icmp6CreateFile() -> HANDLE {
    let device_name_slice: &[u16] = &[
        0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0049, 0x0070, 0x0036,
        0x0000,
    ];
    let device_name_buffer = device_name_slice.as_ptr() as *mut u16;
    let DeviceName = UNICODE_STRING {
        Length: ((device_name_slice.len() - 1) * 2) as u16,
        MaximumLength: (device_name_slice.len() * 2) as u16,
        Buffer: device_name_buffer,
    };

    let mut IcmpFile: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();

    ObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of!(DeviceName) as *mut UNICODE_STRING;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let Status = NtCreateFile(
        &mut IcmpFile,
        GENERIC_EXECUTE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN_IF,
        0,
        core::ptr::null_mut(),
        0,
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return INVALID_HANDLE_VALUE;
    }

    IcmpFile
}

// ---- Icmp6ParseReplies (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn Icmp6ParseReplies(
    ReplyBuffer: LPVOID,
    ReplySize: u32,
) -> u32 {
    if ReplyBuffer.is_null() || ReplySize == 0 {
        return 0;
    }

    let p_echo = ReplyBuffer as PICMPV6_ECHO_REPLY;

    if (*p_echo).Status == IP_SUCCESS {
        return 1;
    }

    SetLastError((*p_echo).Status);
    0
}

// ---- Icmp6SendEcho2 (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn Icmp6SendEcho2(
    IcmpHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    SourceAddress: *mut sockaddr_in6,
    DestinationAddress: *mut sockaddr_in6,
    RequestData: LPVOID,
    RequestSizeArg: u16,
    RequestOptions: *mut IP_OPTION_INFORMATION,
    ReplyBuffer: LPVOID,
    ReplySize: u32,
    Timeout: u32,
) -> u32 {
    let reply_size_full = ReplySize;
    let min_size = (core::mem::size_of::<ICMPV6_ECHO_REPLY>() + core::mem::size_of::<IO_STATUS_BLOCK>()) as u32;
    if reply_size_full < min_size {
        SetLastError(IP_BUF_TOO_SMALL);
        return 0;
    }
    let mut reply_size = reply_size_full - core::mem::size_of::<IO_STATUS_BLOCK>() as u32;
    let io_status_block = (ReplyBuffer as *mut u8).add(reply_size as usize) as *mut IO_STATUS_BLOCK;

    let input_buffer_length = core::mem::size_of::<ICMPV6_ECHO_REQUEST>() as u32 + RequestSizeArg as u32;

    let input_buffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, ((input_buffer_length) as SIZE_T)) as *mut u8;
    if input_buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    let request = input_buffer as *mut ICMPV6_ECHO_REQUEST;

    (*request).DestinationAddress.sin6_port = (*DestinationAddress).sin6_port;
    (*request).DestinationAddress.sin6_flowinfo = (*DestinationAddress).sin6_flowinfo;
    core::ptr::copy_nonoverlapping(
        &(*DestinationAddress).sin6_addr as *const _ as *const u8,
        core::ptr::addr_of_mut!((*request).DestinationAddress.sin6_addr) as *mut u8,
        core::mem::size_of::<in6_addr>(),
    );
    (*request).DestinationAddress.sin6_scope_id = *(*DestinationAddress).__bindgen_anon_1.sin6_scope_id.as_ref();

    (*request).SourceAddress.sin6_port = (*SourceAddress).sin6_port;
    (*request).SourceAddress.sin6_flowinfo = (*SourceAddress).sin6_flowinfo;
    core::ptr::copy_nonoverlapping(
        &(*SourceAddress).sin6_addr as *const _ as *const u8,
        core::ptr::addr_of_mut!((*request).SourceAddress.sin6_addr) as *mut u8,
        core::mem::size_of::<in6_addr>(),
    );
    (*request).SourceAddress.sin6_scope_id = *(*SourceAddress).__bindgen_anon_1.sin6_scope_id.as_ref();

    (*request).Unknown1 = (0x72u32 as u16);
    (*request).Timeout = Timeout;
    (*request).Ttl = ((*RequestOptions).Ttl as u16);
    (*request).Flags = ((*RequestOptions).Flags as u32);

    if RequestSizeArg > 0 {
        core::ptr::copy_nonoverlapping(
            RequestData as *const u8,
            input_buffer.add(core::mem::size_of::<ICMPV6_ECHO_REQUEST>()) as *mut u8,
            RequestSizeArg as usize,
        );
    }

    let h_event: HANDLE = if Event.is_null() && ApcRoutine.is_none() {
        CreateEventW(core::ptr::null_mut(), 0i32, 0i32, core::ptr::null())
    } else {
        Event
    };

    let mut status = NtDeviceIoControlFile(
        IcmpHandle,
        h_event,
        ApcRoutine,
        ApcContext,
        io_status_block,
        IOCTL_ICMP_ECHO_REQUEST,
        input_buffer as *mut core::ffi::c_void,
        input_buffer_length,
        ReplyBuffer,
        reply_size,
    );

    if !Event.is_null() || !ApcRoutine.is_none() {
        SetLastError(RtlNtStatusToDosError(status));
        HeapFree(GetProcessHeap(), 0u32, input_buffer as *mut core::ffi::c_void);
        return 0;
    }

    if status == STATUS_PENDING {
        status = NtWaitForSingleObject(h_event, 0u8, core::ptr::null_mut());
        if NT_SUCCESS(status) {
            status = (*io_status_block).__bindgen_anon_1.Status;
        }
    }

    CloseHandle(h_event);
    HeapFree(GetProcessHeap(), 0u32, input_buffer as *mut core::ffi::c_void);

    if !NT_SUCCESS(status) {
        SetLastError(RtlNtStatusToDosError(status));
        return 0;
    }

    let reply_status = (*(ReplyBuffer as *mut ICMPV6_ECHO_REPLY)).Status;
    if reply_status != IP_SUCCESS {
        SetLastError(reply_status);
        return 0;
    }

    1
}

// ---- IcmpCloseHandle (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn IcmpCloseHandle(IcmpHandle: HANDLE) -> BOOL {
    let status = NtClose(IcmpHandle);
    if !NT_SUCCESS(status) {
        SetLastError(RtlNtStatusToDosError(status));
        return 0;
    }
    1
}

// ---- IcmpCreateFile (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn IcmpCreateFile() -> HANDLE {
    // Build literal wide string for L"\\Device\\Ip"
    let device_name_wide: [u16; 11] = [
        0x005C, // '\'
        0x0044, // 'D'
        0x0065, // 'e'
        0x0076, // 'v'
        0x0069, // 'i'
        0x0063, // 'c'
        0x0065, // 'e'
        0x005C, // '\'
        0x0049, // 'I'
        0x0070, // 'p'
        0x0000, // null terminator
    ];
    let mut device_name = UNICODE_STRING {
        Length: 20,          // 10 chars * 2
        MaximumLength: 22,   // (10+1) * 2
        Buffer: device_name_wide.as_ptr() as *mut u16,
    };

    let mut object_attributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
    object_attributes.RootDirectory = core::ptr::null_mut();
    object_attributes.Attributes = OBJ_CASE_INSENSITIVE;
    object_attributes.ObjectName = &raw mut device_name;
    object_attributes.SecurityDescriptor = core::ptr::null_mut();
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut icmp_file: HANDLE = core::ptr::null_mut();
    let mut io_status_block: IO_STATUS_BLOCK = core::mem::zeroed();

    let status = NtCreateFile(
        &raw mut icmp_file,                     // FileHandle
        GENERIC_EXECUTE,                        // DesiredAccess
        &raw mut object_attributes,            // ObjectAttributes
        &raw mut io_status_block,              // IoStatusBlock
        core::ptr::null_mut(),                 // AllocationSize
        FILE_ATTRIBUTE_NORMAL,                 // FileAttributes
        FILE_SHARE_READ | FILE_SHARE_WRITE,    // ShareAccess
        FILE_OPEN_IF,                          // CreateDisposition
        0,                                     // CreateOptions
        core::ptr::null_mut(),                 // EaBuffer
        0,                                     // EaLength
    );

    if !NT_SUCCESS(status) {
        SetLastError(RtlNtStatusToDosError(status));
        return INVALID_HANDLE_VALUE;
    }

    icmp_file
}

// ---- IcmpSendEcho2 (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn IcmpSendEcho2(
    IcmpHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    DestinationAddress: IPAddr,
    RequestData: LPVOID,
    RequestSizeArg: u16,
    RequestOptions: PIP_OPTION_INFORMATION,
    ReplyBuffer: LPVOID,
    ReplySize: DWORD,
    Timeout: DWORD,
) -> DWORD {
    // Validate ReplySize: must accommodate ICMP_ECHO_REPLY + IO_STATUS_BLOCK
    if ReplySize < (core::mem::size_of::<ICMP_ECHO_REPLY>() as u32 + core::mem::size_of::<IO_STATUS_BLOCK>() as u32) {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    // Check data fits in reply
    if ReplySize < RequestSizeArg as u32 + core::mem::size_of::<ICMP_ECHO_REPLY>() as u32 {
        SetLastError(IP_GENERAL_FAILURE);
        return 0;
    }

    // Place IoStatusBlock at the end of ReplyBuffer
    let mut ReplySize = ReplySize;
    let IoStatusBlock = (ReplyBuffer as *mut u8)
        .add(ReplySize as usize - core::mem::size_of::<IO_STATUS_BLOCK>())
        as *mut IO_STATUS_BLOCK;
    ReplySize -= core::mem::size_of::<IO_STATUS_BLOCK>() as u32;

    // Allocate temporary input buffer for the request
    let InputBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (ReplySize as SIZE_T));
    if InputBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    let Request = InputBuffer as *mut ICMP_ECHO_REQUEST;
    (*Request).Address = DestinationAddress;
    (*Request).Timeout = Timeout;
    (*Request).OptionsOffset = core::mem::size_of::<ICMP_ECHO_REQUEST>() as u16;
    (*Request).DataOffset = core::mem::size_of::<ICMP_ECHO_REQUEST>() as u16;

    if !RequestOptions.is_null() {
        (*Request).HasOptions = 1u8; // TRUE
        (*Request).Ttl = (*RequestOptions).Ttl;
        (*Request).Tos = (*RequestOptions).Tos;
        (*Request).Flags = (*RequestOptions).Flags;

        if (*RequestOptions).OptionsSize > 0 {
            (*Request).OptionsSize = (*RequestOptions).OptionsSize;
            (*Request).DataOffset += ((*Request).OptionsSize as u16);
            CopyMemory(
                (InputBuffer as *mut u8).add(core::mem::size_of::<ICMP_ECHO_REQUEST>()) as *mut c_void,
                (*RequestOptions).OptionsData as *const c_void,
                (*Request).OptionsSize as usize,
            );
        }
    }

    if RequestSizeArg > 0 {
        (*Request).DataSize = (RequestSizeArg as u16);
        CopyMemory(
            (InputBuffer as *mut u8).add((*Request).DataOffset as usize) as *mut c_void,
            RequestData,
            RequestSizeArg as usize,
        );
    }

    // Handle event / APC mode
    let hEvent: HANDLE;
    if Event.is_null() && ApcRoutine.is_none() {
        hEvent = CreateEventW(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
    } else {
        hEvent = Event;
    }

    let Status = NtDeviceIoControlFile(
        IcmpHandle,
        hEvent,
        ApcRoutine,
        ApcContext,
        IoStatusBlock,
        IOCTL_ICMP_ECHO_REQUEST,
        InputBuffer,
        ReplySize,
        ReplyBuffer,
        ReplySize,
    );

    // If caller wants async, return now
    if !Event.is_null() || ApcRoutine.is_some() {
        SetLastError(RtlNtStatusToDosError(Status));
        HeapFree(GetProcessHeap(), 0, InputBuffer);
        return 0;
    }

    // Synchronous path
    let mut Status = Status;
    if Status == STATUS_PENDING {
        Status = NtWaitForSingleObject(hEvent, 0, core::ptr::null_mut());
        if NT_SUCCESS(Status) {
            Status = (*IoStatusBlock).__bindgen_anon_1.Status;
        }
    }

    CloseHandle(hEvent);
    HeapFree(GetProcessHeap(), 0, InputBuffer);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    // Final status from the reply itself
    let Reply = ReplyBuffer as *mut ICMP_ECHO_REPLY;
    Status = ((*Reply).Status as i32);
    if Status != (IP_SUCCESS as i32) {
        SetLastError((Status as u32));
    }

    let nReplies = (*Reply).Reserved;
    (*Reply).Reserved = 0;
    (nReplies as u32)
}

// ---- IcmpSendEcho (retry run) ----
#[no_mangle]
pub unsafe extern "system" fn IcmpSendEcho(
    IcmpHandle: HANDLE,
    DestinationAddress: IPAddr,
    RequestData: LPVOID,
    RequestSizeArg: u16,
    RequestOptions: *const IP_OPTION_INFORMATION,
    ReplyBuffer: LPVOID,
    ReplySize: DWORD,
    Timeout: DWORD,
) -> DWORD {
    if Timeout == 0 || Timeout == u32::MAX {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if ReplySize < core::mem::size_of::<ICMP_ECHO_REPLY>() as DWORD {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }

    if ReplySize < RequestSizeArg as DWORD + core::mem::size_of::<ICMP_ECHO_REPLY>() as DWORD {
        SetLastError(IP_GENERAL_FAILURE);
        return 0;
    }

    let mut InputBufferLength: u32 =
        core::mem::size_of::<ICMP_ECHO_REQUEST>() as u32 + RequestSizeArg as u32;
    if !RequestOptions.is_null() {
        InputBufferLength += (*RequestOptions).OptionsSize as u32;
    }

    if InputBufferLength < ReplySize {
        InputBufferLength = ReplySize;
    }

    let InputBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (InputBufferLength as SIZE_T));
    if InputBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    let Request: *mut ICMP_ECHO_REQUEST = InputBuffer as *mut ICMP_ECHO_REQUEST;
    (*Request).Address = DestinationAddress;
    (*Request).Timeout = Timeout;
    (*Request).OptionsOffset = (core::mem::size_of::<ICMP_ECHO_REQUEST>() as u32 as u16);
    (*Request).DataOffset = (core::mem::size_of::<ICMP_ECHO_REQUEST>() as u32 as u16);

    if !RequestOptions.is_null() {
        (*Request).HasOptions = (1i32 as u8); // TRUE
        (*Request).Ttl = (*RequestOptions).Ttl;
        (*Request).Tos = (*RequestOptions).Tos;
        (*Request).Flags = (*RequestOptions).Flags;

        if (*RequestOptions).OptionsSize > 0 {
            (*Request).OptionsSize = ((*RequestOptions).OptionsSize as u32 as u8);
            (*Request).DataOffset += ((*RequestOptions).OptionsSize as u32 as u16);

            CopyMemory(
                (InputBuffer as *mut u8).add(core::mem::size_of::<ICMP_ECHO_REQUEST>()) as *mut c_void,
                (*RequestOptions).OptionsData as *const c_void,
                (*Request).OptionsSize as usize,
            );
        }
    }

    if RequestSizeArg > 0 {
        (*Request).DataSize = (RequestSizeArg as u32 as u16);
        CopyMemory(
            (InputBuffer as *mut u8).add((*Request).DataOffset as usize) as *mut c_void,
            RequestData,
            RequestSizeArg as usize,
        );
    }

    let hEvent = CreateEventW(
        core::ptr::null_mut(),
        0i32,
        0i32,
        core::ptr::null(),
    );
    if hEvent.is_null() {
        HeapFree(GetProcessHeap(), 0, InputBuffer);
        return 0;
    }

    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status = NtDeviceIoControlFile(
        IcmpHandle,
        hEvent,
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        IOCTL_ICMP_ECHO_REQUEST,
        InputBuffer,
        InputBufferLength,
        ReplyBuffer,
        ReplySize,
    );

    if Status == STATUS_PENDING {
        Status = NtWaitForSingleObject(hEvent, 0u8, core::ptr::null_mut());

        if NT_SUCCESS(Status as i32) {
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }
    }

    CloseHandle(hEvent);
    HeapFree(GetProcessHeap(), 0, InputBuffer);

    if !NT_SUCCESS(Status as i32) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    let Reply: *mut ICMP_ECHO_REPLY = ReplyBuffer as *mut ICMP_ECHO_REPLY;
    Status = ((*Reply).Status as i32);
    if Status != (IP_SUCCESS as i32) {
        SetLastError((Status as u32));
    }

    let nReplies = (*Reply).Reserved;
    (*Reply).Reserved = 0;

    (nReplies as u32)
}
