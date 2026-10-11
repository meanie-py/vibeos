use crate::eventlog::eventlog::PRINT_RECORD;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_E9A72DDF42EB: [u16; 7] = [0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000];

use core::ffi::c_void;

extern "C" {
    static mut onLiveCD: BOOL;
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ConnectPortHandle: HANDLE = NULL;

static mut MessagePortHandle: HANDLE = NULL;
#[no_mangle]
pub unsafe extern "system" fn PortThreadRoutine(Param: PVOID) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Status = InitLogPort();
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    while NT_SUCCESS!(Status) {
        Status = ProcessPortMessage();
    }

    if !ConnectPortHandle.is_null() {
        NtClose(ConnectPortHandle);
    }

    if !MessagePortHandle.is_null() {
        NtClose(MessagePortHandle);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn InitLogPort() -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut PortName: _UNICODE_STRING = RTL_CONSTANT_STRING!(*ELF_PORT_NAME);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Request: PORT_MESSAGE = core::mem::zeroed();
    let FileName: *const core::ffi::c_char =
        b"base/services/eventlog/logport.c\0"
            .as_ptr() as *const core::ffi::c_char;

    ConnectPortHandle = core::ptr::null_mut();
    MessagePortHandle = core::ptr::null_mut();

    // InitializeObjectAttributes(&ObjectAttributes, &PortName, COMMENT_TOKEN_ECHO____, NULL, NULL), expanded inline
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>() as HANDLE;
    ObjectAttributes.Attributes = 0 as ULONG;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(PortName) as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;

    'byebye: {
        Status = NtCreatePort(
            core::ptr::addr_of_mut!(ConnectPortHandle),
            core::ptr::addr_of_mut!(ObjectAttributes),
            0 as ULONG,
            PORT_MAXIMUM_MESSAGE_LENGTH as ULONG,
            (2 * PAGE_SIZE) as ULONG,
        );
        if !NT_SUCCESS!(Status) {
            if DbgPrint(
                b"(%s:%d) NtCreatePort() failed (Status %lx)\n\0".as_ptr() as PCSTR,
                FileName,
                66 as core::ffi::c_int,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    FileName,
                    66 as core::ffi::c_int,
                );
            }
            break 'byebye;
        }

        Status = NtListenPort(
            ConnectPortHandle,
            core::ptr::addr_of_mut!(Request),
        );
        if !NT_SUCCESS!(Status) {
            if DbgPrint(
                b"(%s:%d) NtListenPort() failed (Status %lx)\n\0".as_ptr() as PCSTR,
                FileName,
                73 as core::ffi::c_int,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    FileName,
                    73 as core::ffi::c_int,
                );
            }
            break 'byebye;
        }

        Status = NtAcceptConnectPort(
            core::ptr::addr_of_mut!(MessagePortHandle),
            ConnectPortHandle as PVOID,
            core::ptr::addr_of_mut!(Request),
            TRUE as BOOLEAN,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(Status) {
            if DbgPrint(
                b"(%s:%d) NtAcceptConnectPort() failed (Status %lx)\n\0".as_ptr() as PCSTR,
                FileName,
                81 as core::ffi::c_int,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    FileName,
                    81 as core::ffi::c_int,
                );
            }
            break 'byebye;
        }

        Status = NtCompleteConnectPort(MessagePortHandle);
        if !NT_SUCCESS!(Status) {
            if DbgPrint(
                b"(%s:%d) NtCompleteConnectPort() failed (Status %lx)\n\0".as_ptr() as PCSTR,
                FileName,
                88 as core::ffi::c_int,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    FileName,
                    88 as core::ffi::c_int,
                );
            }
            break 'byebye;
        }
    }

    // ByeBye:
    if !NT_SUCCESS!(Status) {
        if !ConnectPortHandle.is_null() {
            NtClose(ConnectPortHandle);
        }

        if !MessagePortHandle.is_null() {
            NtClose(MessagePortHandle);
        }
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessPortMessage() -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut SystemLog: PLOGFILE = core::ptr::null_mut();
    let mut Buffer: [UCHAR; PORT_MAXIMUM_MESSAGE_LENGTH as usize] = [0; PORT_MAXIMUM_MESSAGE_LENGTH as usize];
    let mut Message: PELF_API_MSG = core::ptr::addr_of_mut!(Buffer) as *mut UCHAR as PELF_API_MSG;
    let mut ErrorMessage: PIO_ERROR_LOG_MESSAGE = core::ptr::null_mut();
    let mut LogBuffer: PEVENTLOGRECORD = core::ptr::null_mut();
    let mut RecSize: SIZE_T = 0;
    let mut Time: ULONG = 0;
    let mut EventType: USHORT = 0;
    let mut SourceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ComputerName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut dwComputerNameLength: DWORD = 0;
    let mut szComputerName: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as usize] =
        [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];

    DPRINT!(b"ProcessPortMessage() called\n\0".as_ptr() as PCSTR);

    SystemLog = LogfListItemByName(RUSTCONV_WSTR_E9A72DDF42EB.as_ptr());

    while TRUE != 0 {
        Status = NtReplyWaitReceivePort(
            MessagePortHandle,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*Message).Header),
        );

        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"NtReplyWaitReceivePort() failed (Status %lx)\n\0".as_ptr() as PCSTR,
                Status,
            );
            break;
        }

        DPRINT!(b"Received message\n\0".as_ptr() as PCSTR);

        if (*Message).Header.u2.s2.Type as u32 == LPC_PORT_CLOSED {
            DPRINT!(b"Port closed\n\0".as_ptr() as PCSTR);
            return STATUS_SUCCESS;
        }

        if (*Message).Header.u2.s2.Type as u32 == LPC_REQUEST {
            DPRINT!(b"Received request\n\0".as_ptr() as PCSTR);
        } else if (*Message).Header.u2.s2.Type as u32 == LPC_DATAGRAM {
            DPRINT!(
                b"Received datagram (0x%x, 0x%x)\n\0".as_ptr() as PCSTR,
                (*Message).Unknown[(0) as usize],
                (*Message).Unknown[(1) as usize],
            );
            ErrorMessage = core::ptr::addr_of_mut!((*Message).IoErrorMessage) as PIO_ERROR_LOG_MESSAGE;

            RtlInitEmptyUnicodeString(
                core::ptr::addr_of_mut!(SourceName),
                core::ptr::null_mut(),
                (0) as USHORT,
            );
            if (*ErrorMessage).DriverNameLength as usize > (core::mem::size_of_val(&UNICODE_NULL)) as usize {
                SourceName.Buffer = ((ErrorMessage as ULONG_PTR)
                    .wrapping_add((*ErrorMessage).DriverNameOffset as ULONG_PTR))
                    as PWSTR;
                SourceName.MaximumLength = (*ErrorMessage).DriverNameLength;
                SourceName.Length = (SourceName.MaximumLength as usize)
                    .wrapping_sub((core::mem::size_of_val(&UNICODE_NULL)) as usize) as USHORT;
            }

            dwComputerNameLength = ARRAYSIZE!(szComputerName) as DWORD;
            if GetComputerNameW(
                core::ptr::addr_of_mut!(szComputerName) as LPWSTR,
                core::ptr::addr_of_mut!(dwComputerNameLength),
            ) == 0
            {
                szComputerName[(0) as usize] = 0x0000u16;
            }

            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(ComputerName),
                core::ptr::addr_of_mut!(szComputerName) as *mut WCHAR as PCWSTR,
            );

            RtlTimeToSecondsSince1970(
                core::ptr::addr_of_mut!((*ErrorMessage).TimeStamp),
                core::ptr::addr_of_mut!(Time),
            );

            /* Set the event type based on the error code severity */
            EventType = ((*ErrorMessage).EntryData.ErrorCode >> 30) as USHORT;
            if EventType as u32 == STATUS_SEVERITY_SUCCESS {
                EventType = EVENTLOG_SUCCESS as USHORT;
            } else if EventType as u32 == STATUS_SEVERITY_INFORMATIONAL {
                // NT_INFORMATION
                EventType = EVENTLOG_INFORMATION_TYPE as USHORT;
            } else if EventType as u32 == STATUS_SEVERITY_WARNING {
                // NT_WARNING
                EventType = EVENTLOG_WARNING_TYPE as USHORT;
            } else if EventType as u32 == STATUS_SEVERITY_ERROR {
                // NT_ERROR
                EventType = EVENTLOG_ERROR_TYPE as USHORT;
            } else {
                /* Unknown severity, set to error */
                EventType = EVENTLOG_ERROR_TYPE as USHORT;
            }

            /*
             * The data being saved consists of the IO_ERROR_LOG_PACKET structure
             * header, plus the additional raw data from the driver.
             */
            LogBuffer = LogfAllocAndBuildNewRecord(
                core::ptr::addr_of_mut!(RecSize),
                Time,
                EventType,
                (*ErrorMessage).EntryData.EventCategory,
                (*ErrorMessage).EntryData.ErrorCode as ULONG,
                core::ptr::addr_of_mut!(SourceName),
                core::ptr::addr_of_mut!(ComputerName),
                (0) as ULONG,
                core::ptr::null_mut(),
                (*ErrorMessage).EntryData.NumberOfStrings,
                ((ErrorMessage as ULONG_PTR)
                    .wrapping_add((*ErrorMessage).EntryData.StringOffset as ULONG_PTR))
                    as PWSTR,
                (core::mem::offset_of!(IO_ERROR_LOG_PACKET, DumpData)
                    + (*ErrorMessage).EntryData.DumpDataSize as usize) as ULONG,
                core::ptr::addr_of_mut!((*ErrorMessage).EntryData) as PVOID,
            );
            if LogBuffer.is_null() {
                DPRINT1!(b"LogfAllocAndBuildNewRecord failed!\n\0".as_ptr() as PCSTR);
                // return STATUS_NO_MEMORY;
                continue;
            }

            if onLiveCD == 0 && !SystemLog.is_null() {
                Status = LogfWriteRecord(SystemLog, LogBuffer, RecSize);
                if !NT_SUCCESS!(Status) {
                    DPRINT1!(
                        b"ERROR writing to event log `%S' (Status 0x%08lx)\n\0".as_ptr() as PCSTR,
                        (*SystemLog).LogName,
                        Status,
                    );
                }
            } else {
                DPRINT1!(b"\n--- EVENTLOG RECORD ---\n\0".as_ptr() as PCSTR);
                PRINT_RECORD(LogBuffer);
                DPRINT1!(b"\n\0".as_ptr() as PCSTR);
            }

            LogfFreeRecord(LogBuffer);
        }
    }

    return Status;
}
