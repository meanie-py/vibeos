use ntdll_sys::nt::*;
// NtCurrentPeb comes from ntdll_sys::nt (FORCEINLINE, NOT an ntdll
// export — a local extern would fail to link). RtlMoveMemory /
// RtlMoveMemory is a real ntdll export.
extern "system" {
    fn RtlMoveMemory(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize);
}
// RtlGetProcessHeap is a C macro (ndk/rtlfuncs.h:1127:
// `#define RtlGetProcessHeap() (NtCurrentPeb()->ProcessHeap)`), NOT an
// ntdll export — declaring it `extern` produces undefined `_RtlGetProcessHeap@0`.
#[inline]
unsafe fn RtlGetProcessHeap() -> HANDLE {
    (*NtCurrentPeb()).ProcessHeap as HANDLE
}
// C: #define CSR_PORT_NAME L"ApiPort" — bindgen mangled the wide
// literal to a narrow `&[u8;8]`; the real type is PCWSTR.
static CSR_PORT_NAME_W: [u16; 8] = [
    b'A' as u16,
    b'p' as u16,
    b'i' as u16,
    b'P' as u16,
    b'o' as u16,
    b'r' as u16,
    b't' as u16,
    0,
];

// `CsrApiPort` is a per-binary CSR API port handle: C declares it once in
// csrlib/connect.c (ntdll side) AND once in csrsrv/init.c (csrsrv.dll side)
// — distinct globals in distinct binaries. In this single Rust staticlib the
// canonical `#[no_mangle]` definition lives in csrsrv/init.rs (one symbol can
// be defined once per crate). Each DLL links libsmss_csrss_rust.a
// independently, so it still gets its own copy of `_CsrApiPort`. Here we
// consume it via the ntdll-sys bindgen `extern { static mut CsrApiPort }`
// (in scope through `use ntdll_sys::nt::*`).
// #[no_mangle] pub static mut CsrApiPort: HANDLE = core::ptr::null_mut();
//
// EXCEPTION — the `csrlib_only` lean build (ntdll.dll client flip): there
// csrsrv/init.rs is gated OUT of the archive, so nobody else defines
// `_CsrApiPort` and the loader symbol would be undefined. csrlib supplies
// its own copy here, exactly as the C connect.c did (`HANDLE CsrApiPort;`).
// In every other build csrsrv/init.rs owns it and this stays OFF (no dup).
#[cfg(feature = "csrlib_only")]
#[no_mangle]
pub static mut CsrApiPort: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut CsrProcessId: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut CsrPortHeap: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut CsrPortMemoryDelta: ULONG_PTR = 0;

#[no_mangle]
pub static mut InsideCsrProcess: BOOLEAN = 0u8;

type PCSR_SERVER_API_ROUTINE = Option<
    unsafe extern "system" fn(Request: PCSR_API_MESSAGE, Reply: PCSR_API_MESSAGE) -> NTSTATUS,
>;

static mut CsrServerApiRoutine: PCSR_SERVER_API_ROUTINE = None;

const SECURITY_NT_AUTHORITY: u64 = 5;
const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC0000106u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as NTSTATUS;
const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000007Bu32 as NTSTATUS;
// ntbasedef.h: ((USHORT) 65534). Was 0xFFFC — the bound rejected 2 bytes early.
const UNICODE_STRING_MAX_BYTES: usize = 0xFFFE;

unsafe fn CsrpConnectToServer(ObjectDirectory: PCWSTR) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let PortNameLength: usize;
    let mut PortName: _UNICODE_STRING = core::mem::zeroed();
    let mut CsrSectionViewSize: LARGE_INTEGER;
    let mut CsrSectionHandle: HANDLE = core::ptr::null_mut();
    let mut LpcWrite: PORT_VIEW;
    let mut LpcRead: REMOTE_PORT_VIEW;
    let mut SecurityQos: SECURITY_QUALITY_OF_SERVICE;
    let mut NtSidAuthority: SID_IDENTIFIER_AUTHORITY = core::mem::zeroed();
    let mut SystemSid: PSID = core::ptr::null_mut();
    let mut ConnectionInfo: CSR_API_CONNECTINFO = core::mem::zeroed();
    let mut ConnectionInfoLength: ULONG = core::mem::size_of::<CSR_API_CONNECTINFO>() as ULONG;

    let mut dir_len = 0usize;
    let mut ptr = ObjectDirectory;
    while *ptr != 0 {
        dir_len += 1;
        ptr = ptr.add(1);
    }

    // C: sizeof(CSR_PORT_NAME) == sizeof(L"ApiPort") == 8 WCHAR == 16 bytes.
    // bindgen mangled CSR_PORT_NAME to a narrow &[u8;8] (8 bytes), so
    // size_of_val(CSR_PORT_NAME) under-counts by 8 → buffer too short →
    // RtlAppendUnicodeToString can't fit "ApiPort" → STATUS_OBJECT_NAME_INVALID
    // at connect → critical client dies → 0xc000021a. Use the wide [u16;8].
    PortNameLength =
        ((dir_len + 1) * core::mem::size_of::<u16>()) + core::mem::size_of_val(&CSR_PORT_NAME_W);
    if PortNameLength > UNICODE_STRING_MAX_BYTES {
        return STATUS_NAME_TOO_LONG;
    }

    PortName.Length = 0;
    PortName.MaximumLength = PortNameLength as u16;

    PortName.Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        PortNameLength as SIZE_T,
    ) as *mut u16;
    if PortName.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlAppendUnicodeToString(&mut PortName, ObjectDirectory);
    static BACKSLASH: [u16; 2] = [b'\\' as u16, 0];
    RtlAppendUnicodeToString(&mut PortName, BACKSLASH.as_ptr());
    RtlAppendUnicodeToString(&mut PortName, CSR_PORT_NAME_W.as_ptr());

    CsrSectionViewSize = core::mem::zeroed();
    CsrSectionViewSize.QuadPart = CSR_CSRSS_SECTION_SIZE as i64;
    Status = NtCreateSection(
        &mut CsrSectionHandle,
        SECTION_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut CsrSectionViewSize,
        PAGE_READWRITE,
        SEC_RESERVE,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    LpcWrite = core::mem::zeroed();
    LpcWrite.Length = core::mem::size_of::<PORT_VIEW>() as u32;
    LpcWrite.SectionHandle = CsrSectionHandle;
    LpcWrite.SectionOffset = 0;
    LpcWrite.ViewSize = CsrSectionViewSize.__bindgen_anon_1.LowPart as SIZE_T;
    LpcWrite.ViewBase = core::ptr::null_mut();
    LpcWrite.ViewRemoteBase = core::ptr::null_mut();

    LpcRead = core::mem::zeroed();
    LpcRead.Length = core::mem::size_of::<REMOTE_PORT_VIEW>() as u32;
    LpcRead.ViewSize = 0;
    LpcRead.ViewBase = core::ptr::null_mut();

    SecurityQos = core::mem::zeroed();
    SecurityQos.ImpersonationLevel = SecurityImpersonation;
    SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING as u8;
    SecurityQos.EffectiveOnly = 1u8;

    ConnectionInfo.DebugFlags = 0;

    NtSidAuthority.Value[5] = SECURITY_NT_AUTHORITY as u8;
    Status = RtlAllocateAndInitializeSid(
        &mut NtSidAuthority,
        1,
        SECURITY_LOCAL_SYSTEM_RID,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut SystemSid,
    );
    if Status < 0 {
        NtClose(CsrSectionHandle);
        return Status;
    }

    Status = NtSecureConnectPort(
        &raw mut CsrApiPort,
        &mut PortName,
        &mut SecurityQos,
        &mut LpcWrite,
        SystemSid,
        &mut LpcRead,
        core::ptr::null_mut(),
        &mut ConnectionInfo as *mut _ as *mut core::ffi::c_void,
        &mut ConnectionInfoLength,
    );
    RtlFreeSid(SystemSid);
    NtClose(CsrSectionHandle);
    if Status < 0 {
        return Status;
    }

    CsrPortMemoryDelta = LpcWrite.ViewRemoteBase as ULONG_PTR - LpcWrite.ViewBase as ULONG_PTR;

    CsrProcessId = ConnectionInfo.ServerProcessId;

    let peb = NtCurrentPeb() as *mut _PEB;
    (*peb).ReadOnlySharedMemoryBase = ConnectionInfo.SharedSectionBase;
    (*peb).ReadOnlySharedMemoryHeap = ConnectionInfo.SharedSectionHeap;
    (*peb).ReadOnlyStaticServerData =
        ConnectionInfo.SharedStaticServerData as *mut *mut core::ffi::c_void;

    CsrPortHeap = RtlCreateHeap(
        0,
        LpcWrite.ViewBase,
        LpcWrite.ViewSize,
        PAGE_SIZE as SIZE_T,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if CsrPortHeap.is_null() {
        NtClose(CsrApiPort);
        CsrApiPort = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrClientConnectToServer(
    ObjectDirectory: PCWSTR,
    ServerId: ULONG,
    ConnectionInfo: PVOID,
    ConnectionInfoSize: *mut ULONG,
    ServerToServerCall: PBOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let NtHeader: PIMAGE_NT_HEADERS;

    if !ConnectionInfo.is_null() && (ConnectionInfoSize.is_null() || *ConnectionInfoSize == 0) {
        return STATUS_INVALID_PARAMETER;
    }

    if InsideCsrProcess != 0 {
        if !ServerToServerCall.is_null() {
            *ServerToServerCall = 1u8;
        }
        return STATUS_SUCCESS;
    }

    let peb = NtCurrentPeb() as *mut _PEB;
    NtHeader = RtlImageNtHeader((*peb).ImageBaseAddress);
    if NtHeader.is_null() {
        return STATUS_INVALID_IMAGE_FORMAT;
    }
    InsideCsrProcess = if (*NtHeader).OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_NATIVE as u16 {
        1u8
    } else {
        0u8
    };

    if InsideCsrProcess != 0 {
        let mut CsrSrvName: _UNICODE_STRING = core::mem::zeroed();
        let mut hCsrSrv: HANDLE = core::ptr::null_mut();
        let mut CsrServerRoutineName: ANSI_STRING = core::mem::zeroed();

        static CSRSRV: [u16; 7] = [
            b'c' as u16,
            b's' as u16,
            b'r' as u16,
            b's' as u16,
            b'r' as u16,
            b'v' as u16,
            0,
        ];
        RtlInitUnicodeString(&mut CsrSrvName, CSRSRV.as_ptr());
        Status = LdrGetDllHandle(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut CsrSrvName,
            &mut hCsrSrv,
        );
        if Status < 0 {
            InsideCsrProcess = 0u8;
            // goto ClientConnect
        } else {
            RtlInitAnsiString(
                &mut CsrServerRoutineName,
                b"CsrCallServerFromServer\0".as_ptr() as *const i8,
            );
            Status = LdrGetProcedureAddress(
                hCsrSrv,
                &mut CsrServerRoutineName,
                0,
                &raw mut CsrServerApiRoutine as *mut _ as *mut PVOID,
            );

            CsrPortHeap = RtlGetProcessHeap();

            if !ServerToServerCall.is_null() {
                *ServerToServerCall = InsideCsrProcess;
            }
            return STATUS_SUCCESS;
        }
    }

    // ClientConnect:
    if !ConnectionInfo.is_null() {
        let mut ApiMessage: CSR_API_MESSAGE = core::mem::zeroed();
        let ClientConnect: PCSR_CLIENT_CONNECT = &mut ApiMessage
            .__bindgen_anon_1
            .__bindgen_anon_1
            .Data
            .CsrClientConnect as *mut _;
        let CaptureBuffer: PCSR_CAPTURE_BUFFER;

        InsideCsrProcess = 0u8;

        if CsrApiPort.is_null() {
            Status = CsrpConnectToServer(ObjectDirectory);
            if Status < 0 {
                return Status;
            }
        }

        (*ClientConnect).ServerId = ServerId;
        (*ClientConnect).ConnectionInfoSize = *ConnectionInfoSize;

        CaptureBuffer = CsrAllocateCaptureBuffer(1, (*ClientConnect).ConnectionInfoSize);
        if CaptureBuffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        CsrCaptureMessageBuffer(
            CaptureBuffer,
            ConnectionInfo,
            (*ClientConnect).ConnectionInfoSize,
            &mut (*ClientConnect).ConnectionInfo as *mut _ as *mut PVOID,
        );

        *ConnectionInfoSize = (*ClientConnect).ConnectionInfoSize;

        Status = CsrClientCallServer(
            &mut ApiMessage,
            CaptureBuffer,
            ((CSRSRV_SERVERDLL_INDEX << 16) | CsrpClientConnect) as CSR_API_NUMBER,
            core::mem::size_of::<CSR_CLIENT_CONNECT>() as ULONG,
        );

        RtlMoveMemory(
            ConnectionInfo,
            (*ClientConnect).ConnectionInfo as *const core::ffi::c_void,
            *ConnectionInfoSize as usize,
        );

        CsrFreeCaptureBuffer(CaptureBuffer);
    } else {
        Status = STATUS_SUCCESS;
    }

    if !ServerToServerCall.is_null() {
        *ServerToServerCall = InsideCsrProcess;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrClientCallServer(
    ApiMessage: PCSR_API_MESSAGE,
    CaptureBuffer: PCSR_CAPTURE_BUFFER,
    ApiNumber: CSR_API_NUMBER,
    DataLength: ULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if DataLength > (MAXSHORT as ULONG - core::mem::size_of::<CSR_API_MESSAGE>() as ULONG) {
        return STATUS_INVALID_PARAMETER;
    }

    (*ApiMessage).Header.u2.ZeroInit = 0;
    // DataLength/TotalLength are CSHORT (i16) in PORT_MESSAGE.
    (*ApiMessage).Header.u1.s1.DataLength = ((DataLength as u16).wrapping_add(
        core::mem::offset_of!(CSR_API_MESSAGE, __bindgen_anon_1.__bindgen_anon_1.Data) as u16
            - core::mem::size_of_val(&(*ApiMessage).Header) as u16,
    )) as i16;
    (*ApiMessage).Header.u1.s1.TotalLength = ((DataLength as u16).wrapping_add(
        core::mem::size_of::<CSR_API_MESSAGE>() as u16
            - core::mem::size_of_val(&(*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Data) as u16,
    )) as i16;

    (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.ApiNumber = ApiNumber;
    (*ApiMessage)
        .__bindgen_anon_1
        .__bindgen_anon_1
        .CsrCaptureData = core::ptr::null_mut();

    if InsideCsrProcess == 0 {
        let mut PointerCount: ULONG;
        let mut OffsetPointer: *mut ULONG_PTR;

        if !CaptureBuffer.is_null() {
            (*ApiMessage)
                .__bindgen_anon_1
                .__bindgen_anon_1
                .CsrCaptureData =
                (CaptureBuffer as ULONG_PTR + CsrPortMemoryDelta) as PCSR_CAPTURE_BUFFER;

            (*CaptureBuffer).BufferEnd = core::ptr::null_mut();

            PointerCount = (*CaptureBuffer).PointerCount;
            OffsetPointer = (*CaptureBuffer).PointerOffsetsArray.as_mut_ptr();
            while PointerCount > 0 {
                if *OffsetPointer != 0 {
                    *((*OffsetPointer) as *mut ULONG_PTR) =
                        (*((*OffsetPointer) as *mut ULONG_PTR)).wrapping_add(CsrPortMemoryDelta);
                    *OffsetPointer = (*OffsetPointer).wrapping_sub(ApiMessage as ULONG_PTR);
                }
                OffsetPointer = OffsetPointer.add(1);
                PointerCount -= 1;
            }
        }

        Status = NtRequestWaitReplyPort(
            CsrApiPort,
            &mut (*ApiMessage).Header,
            &mut (*ApiMessage).Header,
        );

        if !CaptureBuffer.is_null() {
            (*ApiMessage)
                .__bindgen_anon_1
                .__bindgen_anon_1
                .CsrCaptureData = (((*ApiMessage)
                .__bindgen_anon_1
                .__bindgen_anon_1
                .CsrCaptureData as ULONG_PTR)
                .wrapping_sub(CsrPortMemoryDelta))
                as PCSR_CAPTURE_BUFFER;

            PointerCount = (*CaptureBuffer).PointerCount;
            OffsetPointer = (*CaptureBuffer).PointerOffsetsArray.as_mut_ptr();
            while PointerCount > 0 {
                if *OffsetPointer != 0 {
                    *OffsetPointer = (*OffsetPointer).wrapping_add(ApiMessage as ULONG_PTR);
                    *((*OffsetPointer) as *mut ULONG_PTR) =
                        (*((*OffsetPointer) as *mut ULONG_PTR)).wrapping_sub(CsrPortMemoryDelta);
                }
                OffsetPointer = OffsetPointer.add(1);
                PointerCount -= 1;
            }
        }

        if Status < 0 {
            (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = Status;
        }
    } else {
        (*ApiMessage).Header.__bindgen_anon_1.ClientId = (*NtCurrentTeb()).ClientId;

        Status = (CsrServerApiRoutine.unwrap())(ApiMessage, ApiMessage);

        if Status < 0 {
            (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = Status;
        }
    }

    (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrGetProcessId() -> HANDLE {
    CsrProcessId
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Client/Server Runtime SubSystem
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     CSR Client Library - CSR connection and calling
//  * COPYRIGHT:   Copyright 2005-2013 Alex Ionescu <alex@relsoft.net>
//  *              Copyright 2012-2022 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "csrlib.h"
//
// #define NTOS_MODE_USER
// #include <ndk/ldrfuncs.h>
// #include <ndk/lpcfuncs.h>
// #include <ndk/mmfuncs.h>
// #include <ndk/obfuncs.h>
// #include <ndk/umfuncs.h>
//
// #include <csrsrv.h> // For CSR_CSRSS_SECTION_SIZE
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// HANDLE CsrApiPort;
// HANDLE CsrProcessId;
// HANDLE CsrPortHeap;
// ULONG_PTR CsrPortMemoryDelta;
// BOOLEAN InsideCsrProcess = FALSE;
//
// typedef NTSTATUS
// (NTAPI *PCSR_SERVER_API_ROUTINE)(
//     _In_ PCSR_API_MESSAGE Request,
//     _Inout_ PCSR_API_MESSAGE Reply);
//
// PCSR_SERVER_API_ROUTINE CsrServerApiRoutine;
//
// /* FUNCTIONS ******************************************************************/
//
// static NTSTATUS
// CsrpConnectToServer(
//     _In_ PCWSTR ObjectDirectory)
// {
//     NTSTATUS Status;
//     SIZE_T PortNameLength;
//     UNICODE_STRING PortName;
//     LARGE_INTEGER CsrSectionViewSize;
//     HANDLE CsrSectionHandle;
//     PORT_VIEW LpcWrite;
//     REMOTE_PORT_VIEW LpcRead;
//     SECURITY_QUALITY_OF_SERVICE SecurityQos;
//     SID_IDENTIFIER_AUTHORITY NtSidAuthority = {SECURITY_NT_AUTHORITY};
//     PSID SystemSid = NULL;
//     CSR_API_CONNECTINFO ConnectionInfo;
//     ULONG ConnectionInfoLength = sizeof(ConnectionInfo);
//
//     DPRINT("%s(%S)\n", __FUNCTION__, ObjectDirectory);
//
//     /* Calculate the total port name size */
//     PortNameLength = ((wcslen(ObjectDirectory) + 1) * sizeof(WCHAR)) +
//                      sizeof(CSR_PORT_NAME);
//     if (PortNameLength > UNICODE_STRING_MAX_BYTES)
//     {
//         DPRINT1("PortNameLength too big: %Iu\n", PortNameLength);
//         return STATUS_NAME_TOO_LONG;
//     }
//
//     /* Set the port name */
//     PortName.Length = 0;
//     PortName.MaximumLength = (USHORT)PortNameLength;
//
//     /* Allocate a buffer for it */
//     PortName.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, PortNameLength);
//     if (PortName.Buffer == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Create the name */
//     RtlAppendUnicodeToString(&PortName, ObjectDirectory);
//     RtlAppendUnicodeToString(&PortName, L"\\");
//     RtlAppendUnicodeToString(&PortName, CSR_PORT_NAME);
//
//     /* Create a section for the port memory */
//     CsrSectionViewSize.QuadPart = CSR_CSRSS_SECTION_SIZE;
//     Status = NtCreateSection(&CsrSectionHandle,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              &CsrSectionViewSize,
//                              PAGE_READWRITE,
//                              SEC_RESERVE,
//                              NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failure allocating CSR Section\n");
//         return Status;
//     }
//
//     /* Set up the port view structures to match them with the section */
//     LpcWrite.Length = sizeof(LpcWrite);
//     LpcWrite.SectionHandle = CsrSectionHandle;
//     LpcWrite.SectionOffset = 0;
//     LpcWrite.ViewSize = CsrSectionViewSize.u.LowPart;
//     LpcWrite.ViewBase = 0;
//     LpcWrite.ViewRemoteBase = 0;
//     LpcRead.Length = sizeof(LpcRead);
//     LpcRead.ViewSize = 0;
//     LpcRead.ViewBase = 0;
//
//     /* Setup the QoS */
//     SecurityQos.ImpersonationLevel = SecurityImpersonation;
//     SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
//     SecurityQos.EffectiveOnly = TRUE;
//
//     /* Setup the connection info */
//     ConnectionInfo.DebugFlags = 0;
//
//     /* Create a SID for us */
//     Status = RtlAllocateAndInitializeSid(&NtSidAuthority,
//                                          1,
//                                          SECURITY_LOCAL_SYSTEM_RID,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          0,
//                                          &SystemSid);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failure */
//         DPRINT1("Couldn't allocate SID\n");
//         NtClose(CsrSectionHandle);
//         return Status;
//     }
//
//     /* Connect to the port */
//     Status = NtSecureConnectPort(&CsrApiPort,
//                                  &PortName,
//                                  &SecurityQos,
//                                  &LpcWrite,
//                                  SystemSid,
//                                  &LpcRead,
//                                  NULL,
//                                  &ConnectionInfo,
//                                  &ConnectionInfoLength);
//     RtlFreeSid(SystemSid);
//     NtClose(CsrSectionHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failure */
//         DPRINT1("Couldn't connect to CSR port\n");
//         return Status;
//     }
//
//     /* Save the delta between the sections, for capture usage later */
//     CsrPortMemoryDelta = (ULONG_PTR)LpcWrite.ViewRemoteBase -
//                          (ULONG_PTR)LpcWrite.ViewBase;
//
//     /* Save the Process */
//     CsrProcessId = ConnectionInfo.ServerProcessId;
//
//     /* Save CSR Section data */
//     NtCurrentPeb()->ReadOnlySharedMemoryBase = ConnectionInfo.SharedSectionBase;
//     NtCurrentPeb()->ReadOnlySharedMemoryHeap = ConnectionInfo.SharedSectionHeap;
//     NtCurrentPeb()->ReadOnlyStaticServerData = ConnectionInfo.SharedStaticServerData;
//
//     /* Create the port heap */
//     CsrPortHeap = RtlCreateHeap(0,
//                                 LpcWrite.ViewBase,
//                                 LpcWrite.ViewSize,
//                                 PAGE_SIZE,
//                                 0,
//                                 0);
//     if (CsrPortHeap == NULL)
//     {
//         /* Failure */
//         DPRINT1("Couldn't create heap for CSR port\n");
//         NtClose(CsrApiPort);
//         CsrApiPort = NULL;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// CsrClientConnectToServer(
//     _In_ PCWSTR ObjectDirectory,
//     _In_ ULONG ServerId,
//     _In_ PVOID ConnectionInfo,
//     _Inout_ PULONG ConnectionInfoSize,
//     _Out_ PBOOLEAN ServerToServerCall)
// {
//     NTSTATUS Status;
//     PIMAGE_NT_HEADERS NtHeader;
//
//     DPRINT("CsrClientConnectToServer: %lx %p\n", ServerId, ConnectionInfo);
//
//     /* Validate the Connection Info */
//     if (ConnectionInfo && (!ConnectionInfoSize || !*ConnectionInfoSize))
//     {
//         DPRINT1("Connection info given, but no length\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if we're inside a CSR Process */
//     if (InsideCsrProcess)
//     {
//         /* Tell the client that we're already inside CSR */
//         if (ServerToServerCall) *ServerToServerCall = TRUE;
//         return STATUS_SUCCESS;
//     }
//
//     /*
//      * We might be in a CSR Process but not know it, if this is the first call.
//      * So let's find out.
//      */
//     if (!(NtHeader = RtlImageNtHeader(NtCurrentPeb()->ImageBaseAddress)))
//     {
//         /* The image isn't valid */
//         DPRINT1("Invalid image\n");
//         return STATUS_INVALID_IMAGE_FORMAT;
//     }
//     InsideCsrProcess = (NtHeader->OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_NATIVE);
//
//     /* Now we can check if we are inside or not */
//     if (InsideCsrProcess)
//     {
//         UNICODE_STRING CsrSrvName;
//         HANDLE hCsrSrv;
//         ANSI_STRING CsrServerRoutineName;
//
//         /* We're inside, so let's find csrsrv */
//         RtlInitUnicodeString(&CsrSrvName, L"csrsrv");
//         Status = LdrGetDllHandle(NULL,
//                                  NULL,
//                                  &CsrSrvName,
//                                  &hCsrSrv);
//         if (!NT_SUCCESS(Status))
//         {
//             /* csrsrv not loaded — we're a native process but not a CSR server.
//              * Fall through to the LPC client connection path. */
//             DPRINT1("Native process but csrsrv not loaded (0x%lx), using LPC\n", Status);
//             InsideCsrProcess = FALSE;
//             goto ClientConnect;
//         }
//
//         /* Now get the Server to Server routine */
//         RtlInitAnsiString(&CsrServerRoutineName, "CsrCallServerFromServer");
//         Status = LdrGetProcedureAddress(hCsrSrv,
//                                         &CsrServerRoutineName,
//                                         0L,
//                                         (PVOID*)&CsrServerApiRoutine);
//
//         /* Use the local heap as port heap */
//         CsrPortHeap = RtlGetProcessHeap();
//
//         /* Tell the caller we're inside the server */
//         if (ServerToServerCall) *ServerToServerCall = InsideCsrProcess;
//         return STATUS_SUCCESS;
//     }
//
// ClientConnect:
//     /* Now check if connection info is given */
//     if (ConnectionInfo)
//     {
//         CSR_API_MESSAGE ApiMessage;
//         PCSR_CLIENT_CONNECT ClientConnect = &ApiMessage.Data.CsrClientConnect;
//         PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//         /* Well, we're definitely in a client now */
//         InsideCsrProcess = FALSE;
//
//         /* Do we have a connection to CSR yet? */
//         if (!CsrApiPort)
//         {
//             /* No, set it up now */
//             Status = CsrpConnectToServer(ObjectDirectory);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Failed */
//                 DPRINT1("Failure to connect to CSR\n");
//                 return Status;
//             }
//         }
//
//         /* Setup the connect message header */
//         ClientConnect->ServerId = ServerId;
//         ClientConnect->ConnectionInfoSize = *ConnectionInfoSize;
//
//         /* Setup a buffer for the connection info */
//         CaptureBuffer = CsrAllocateCaptureBuffer(1, ClientConnect->ConnectionInfoSize);
//         if (CaptureBuffer == NULL)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Capture the connection info data */
//         CsrCaptureMessageBuffer(CaptureBuffer,
//                                 ConnectionInfo,
//                                 ClientConnect->ConnectionInfoSize,
//                                 &ClientConnect->ConnectionInfo);
//
//         /* Return the allocated length */
//         *ConnectionInfoSize = ClientConnect->ConnectionInfoSize;
//
//         /* Call CSR */
//         Status = CsrClientCallServer(&ApiMessage,
//                                      CaptureBuffer,
//                                      CSR_CREATE_API_NUMBER(CSRSRV_SERVERDLL_INDEX, CsrpClientConnect),
//                                      sizeof(*ClientConnect));
//
//         /* Copy the updated connection info data back into the user buffer */
//         RtlMoveMemory(ConnectionInfo,
//                       ClientConnect->ConnectionInfo,
//                       *ConnectionInfoSize);
//
//         /* Free the capture buffer */
//         CsrFreeCaptureBuffer(CaptureBuffer);
//     }
//     else
//     {
//         /* No connection info, just return */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Let the caller know if this was server to server */
//     DPRINT("Status was: 0x%lx. Are we in server: 0x%x\n", Status, InsideCsrProcess);
//     if (ServerToServerCall) *ServerToServerCall = InsideCsrProcess;
//
//     return Status;
// }
//
// #if 0
// //
// // Structures can be padded at the end, causing the size of the entire structure
// // minus the size of the last field, not to be equal to the offset of the last
// // field.
// //
// typedef struct _TEST_EMBEDDED
// {
//     ULONG One;
//     ULONG Two;
//     ULONG Three;
// } TEST_EMBEDDED;
//
// typedef struct _TEST
// {
//     PORT_MESSAGE h;
//     TEST_EMBEDDED Three;
// } TEST;
//
// C_ASSERT(sizeof(PORT_MESSAGE) == 0x18);
// C_ASSERT(FIELD_OFFSET(TEST, Three) == 0x18);
// C_ASSERT(sizeof(TEST_EMBEDDED) == 0xC);
//
// C_ASSERT(sizeof(TEST) != (sizeof(TEST_EMBEDDED) + sizeof(PORT_MESSAGE)));
// C_ASSERT((sizeof(TEST) - sizeof(TEST_EMBEDDED)) != FIELD_OFFSET(TEST, Three));
// #endif
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// CsrClientCallServer(
//     _Inout_ PCSR_API_MESSAGE ApiMessage,
//     _Inout_opt_ PCSR_CAPTURE_BUFFER CaptureBuffer,
//     _In_ CSR_API_NUMBER ApiNumber,
//     _In_ ULONG DataLength)
// {
//     NTSTATUS Status;
//
//     /* Make sure the length is valid */
//     if (DataLength > (MAXSHORT - sizeof(CSR_API_MESSAGE)))
//     {
//         DPRINT1("DataLength too big: %lu\n", DataLength);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Fill out the Port Message Header */
//     ApiMessage->Header.u2.ZeroInit = 0;
//     /* DataLength = user_data_size + anything between
//      * header and data, including intermediate padding */
//     ApiMessage->Header.u1.s1.DataLength = (CSHORT)DataLength +
//         FIELD_OFFSET(CSR_API_MESSAGE, Data) - sizeof(ApiMessage->Header);
//     /* TotalLength = header_size + DataLength + any structure trailing padding */
//     ApiMessage->Header.u1.s1.TotalLength = (CSHORT)DataLength +
//         sizeof(CSR_API_MESSAGE) - sizeof(ApiMessage->Data);
//
//     /* Fill out the CSR Header */
//     ApiMessage->ApiNumber = ApiNumber;
//     ApiMessage->CsrCaptureData = NULL;
//
//     DPRINT("API: %lx, u1.s1.DataLength: %x, u1.s1.TotalLength: %x\n",
//            ApiNumber,
//            ApiMessage->Header.u1.s1.DataLength,
//            ApiMessage->Header.u1.s1.TotalLength);
//
//     /* Check if we are already inside a CSR Server */
//     if (!InsideCsrProcess)
//     {
//         ULONG PointerCount;
//         PULONG_PTR OffsetPointer;
//
//         /* Check if we got a Capture Buffer */
//         if (CaptureBuffer)
//         {
//             /*
//              * We have to convert from our local (client) view
//              * to the remote (server) view.
//              */
//             ApiMessage->CsrCaptureData = (PCSR_CAPTURE_BUFFER)
//                 ((ULONG_PTR)CaptureBuffer + CsrPortMemoryDelta);
//
//             /* Lock the buffer */
//             CaptureBuffer->BufferEnd = NULL;
//
//             /*
//              * Each client pointer inside the CSR message is converted into
//              * a server pointer, and each pointer to these message pointers
//              * is converted into an offset.
//              */
//             PointerCount  = CaptureBuffer->PointerCount;
//             OffsetPointer = CaptureBuffer->PointerOffsetsArray;
//             while (PointerCount--)
//             {
//                 if (*OffsetPointer != 0)
//                 {
//                     *(PULONG_PTR)*OffsetPointer += CsrPortMemoryDelta;
//                     *OffsetPointer -= (ULONG_PTR)ApiMessage;
//                 }
//                 ++OffsetPointer;
//             }
//         }
//
//         /* Send the LPC Message */
//         Status = NtRequestWaitReplyPort(CsrApiPort,
//                                         &ApiMessage->Header,
//                                         &ApiMessage->Header);
//
//         /* Check if we got a Capture Buffer */
//         if (CaptureBuffer)
//         {
//             /*
//              * We have to convert back from the remote (server) view
//              * to our local (client) view.
//              */
//             ApiMessage->CsrCaptureData = (PCSR_CAPTURE_BUFFER)
//                 ((ULONG_PTR)ApiMessage->CsrCaptureData - CsrPortMemoryDelta);
//
//             /*
//              * Convert back the offsets into pointers to CSR message
//              * pointers, and convert back these message server pointers
//              * into client pointers.
//              */
//             PointerCount  = CaptureBuffer->PointerCount;
//             OffsetPointer = CaptureBuffer->PointerOffsetsArray;
//             while (PointerCount--)
//             {
//                 if (*OffsetPointer != 0)
//                 {
//                     *OffsetPointer += (ULONG_PTR)ApiMessage;
//                     *(PULONG_PTR)*OffsetPointer -= CsrPortMemoryDelta;
//                 }
//                 ++OffsetPointer;
//             }
//         }
//
//         /* Check for success */
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed. Overwrite the return value with the failure. */
//             DPRINT1("LPC Failed: %lx\n", Status);
//             ApiMessage->Status = Status;
//         }
//     }
//     else
//     {
//         /* This is a server-to-server call */
//         DPRINT("Server-to-server call\n");
//
//         /* Save our CID; we check this equality inside CsrValidateMessageBuffer */
//         ApiMessage->Header.ClientId = NtCurrentTeb()->ClientId;
//
//         /* Do a direct call */
//         Status = CsrServerApiRoutine(ApiMessage, ApiMessage);
//
//         /* Check for success */
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed. Overwrite the return value with the failure. */
//             ApiMessage->Status = Status;
//         }
//     }
//
//     /* Return the CSR Result */
//     DPRINT("Got back: 0x%lx\n", ApiMessage->Status);
//     return ApiMessage->Status;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// CsrGetProcessId(VOID)
// {
//     return CsrProcessId;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
