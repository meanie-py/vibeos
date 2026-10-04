use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    MmGetPhysicalAddress, PHYSICAL_ADDRESS, PKD_CONTEXT, PLOADER_PARAMETER_BLOCK, PSTRING,
};
use core::ffi::c_void;

// SharedUserData is the C macro `((KUSER_SHARED_DATA*)KI_USER_SHARED_DATA)` — a
// fixed kernel VA, NOT a real symbol (ntkrnl_sys's extern is a kd64 shim absent
// from a standalone transport DLL). Define the fixed address locally. i386
// kernel = 0xFFDF0000; amd64/arm64 = 0xFFFFF78000000000.
#[cfg(target_arch = "x86")]
const SharedUserData: *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA =
    0xFFDF_0000usize as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
const SharedUserData: *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA =
    0xFFFF_F780_0000_0000usize as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;

// VOID is C's void; in FFI contexts we model it as c_void.
type VOID = core::ffi::c_void;

// Header-only types/constants (drivers/base/kdvm/kdvm.h), reconstructed.
use crate::kdvm_h::{
    PFNDBGPRNT, KDRPC_PROTOCOL_VERSION, CONNECTION_TEST_ROUNDS, KDVM_BUFFER_SIZE,
    KDRPC_TEST_BUFFER_SIZE, KDP_STATUS, KDP_PACKET_RESEND, KDVM_CMD_HEADER,
    KDVM_RECEIVE_HEADER, KDVM_SEND_PKT_REQUEST, KDVM_SEND_PKT_RESULT,
    KDVM_RECV_PKT_REQUEST, KDVM_RECV_PKT_RESULT,
};

// kdvbox.c symbols (same DLL, other module).
use crate::kdvbox::{KdVmKdVmExchangeData, KdVmPrepareBuffer};

// STATUS_CONNECTION_REFUSED is not exposed by ntkrnl-sys; define locally.
pub const STATUS_CONNECTION_REFUSED: NTSTATUS = 0xC0000236u32 as NTSTATUS;

// `KDDBGPRINT(...)` — the C macro expands to nothing unless KDDEBUG is defined
// (it is not), so the faithful translation is a no-op that discards its args.
macro_rules! KDDBGPRINT {
    ($($arg:tt)*) => {{ }};
}

// KdDebuggerNotPresent is the ntoskrnl DATA export behind the
// KD_DEBUGGER_NOT_PRESENT macro (kdfuncs.h). Cross-DLL data import → raw-dylib
// so it links on i386 too (a plain extern static won't).
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut KdDebuggerNotPresent: BOOLEAN;
}

// RtlEqualMemory is a C macro `(!memcmp(a,b,n))`, not an export — render it as
// a local inline over memcmp (resolved from libmemcmp at final link).
#[inline]
unsafe fn RtlEqualMemory(s1: *const VOID, s2: *const VOID, len: usize) -> BOOLEAN {
    (memcmp(s1 as *const u8, s2 as *const u8, len) == 0) as BOOLEAN
}

extern "C" {
    fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32;
    fn _strupr(s: PCHAR) -> PCHAR;
    fn strlen(s: PCHAR) -> usize;
    fn strncmp(s1: PCHAR, s2: PCHAR, n: usize) -> i32;
    fn strstr(haystack: PCHAR, needle: PCHAR) -> PCHAR;
}

// ---- file-scope preamble (translated) ----
pub static mut KdVmCmdMagic: [CHAR; 9] = [b'~' as CHAR, b'k' as CHAR, b'd' as CHAR, b'V' as CHAR, b'M' as CHAR, b'v' as CHAR, b'A' as CHAR, b' ' as CHAR, 0];
pub static mut KdVmReplyMagic: [CHAR; 10] = [b'+' as CHAR, b'+' as CHAR, b'k' as CHAR, b'd' as CHAR, b'V' as CHAR, b'M' as CHAR, b'v' as CHAR, b'A' as CHAR, b' ' as CHAR, 0];
pub const KDVM_CMD_TestConnection: UCHAR = b't';
pub const KDVM_CMD_ReceivePacket: UCHAR = b'r';
pub const KDVM_CMD_SendPacket: UCHAR = b's';
pub const KDVM_CMD_VersionReport: UCHAR = b'v';
#[no_mangle]
pub static mut KdVmDataBuffer: [UCHAR; KDVM_BUFFER_SIZE as usize] = [0; KDVM_BUFFER_SIZE as usize];
#[no_mangle]
pub static mut KdVmBufferPhysicalAddress: PHYSICAL_ADDRESS = PHYSICAL_ADDRESS { QuadPart: 0 };
#[no_mangle]
pub static mut KdVmBufferPos: ULONG = 0;
#[no_mangle]
pub static mut KdpDbgPrint: PFNDBGPRNT = None;
unsafe fn KdVmDbgDumpRow(
    Buffer: PUCHAR,
    Size: ULONG) {
    let mut i: ULONG;
    i = 0;
    while i < Size {
        if let Some(f) = KdpDbgPrint {
            f(b"%02x \0".as_ptr() as PCHAR, *Buffer.offset(i as isize) as core::ffi::c_uint);
        }
        i += 1;
    }
    if let Some(f) = KdpDbgPrint {
        f(b"\n\0".as_ptr() as PCHAR);
    }
}
#[no_mangle]
pub unsafe extern "system" fn KdVmDbgDumpBuffer(
    Buffer: PVOID,
    Size: ULONG) {
    let mut CurrentRow: PUCHAR;
    let mut i: ULONG;

    CurrentRow = Buffer as PUCHAR;
    i = 0;
    while i < (Size / 16) {
        KdVmDbgDumpRow(CurrentRow, 16);
        CurrentRow = CurrentRow.offset(16);
        i += 1;
    }
    KdVmDbgDumpRow(CurrentRow, Size % 16 );
}

unsafe fn KdVmAddToBuffer(
    Data: PVOID,
    DataSize: ULONG,
) -> BOOLEAN {
    if ((KdVmBufferPos + DataSize) > KDVM_BUFFER_SIZE) ||
       ((KdVmBufferPos + DataSize) < KdVmBufferPos)
    {
        KDDBGPRINT!(
            b"KdVmAddToBuffer: Buffer overflow! Need %lu, remaining: %lu\n\0".as_ptr() as PCHAR,
            DataSize,
            KDVM_BUFFER_SIZE - KdVmBufferPos,
        );
        return 0u8;
    }

    RtlCopyMemory(
        KdVmDataBuffer.as_mut_ptr().add(KdVmBufferPos as usize) as PVOID,
        Data,
        DataSize as usize,
    );
    KdVmBufferPos += DataSize;
    return 1u8;
}

unsafe fn KdVmAddCommandToBuffer(
    Command: UCHAR,
    Buffer: PVOID,
    BufferSize: SIZE_T,
) -> BOOLEAN {
    let mut Header: KDVM_CMD_HEADER = core::mem::zeroed();

    RtlCopyMemory(
        core::ptr::addr_of_mut!(Header.Magic) as PVOID,
        KdVmCmdMagic.as_ptr() as PVOID,
        core::mem::size_of_val(&Header.Magic),
    );
    Header.Command = Command;

    if KdVmAddToBuffer(
        core::ptr::addr_of_mut!(Header) as PVOID,
        core::mem::size_of::<KDVM_CMD_HEADER>() as ULONG,
    ) == 0u8
    {
        return 0u8;
    }

    if KdVmAddToBuffer(Buffer, BufferSize as ULONG) == 0u8 {
        return 0u8;
    }

    1u8
}
unsafe fn KdVmSendReceive(
    ReceiveDataSize: *mut ULONG
) -> PVOID {
    let mut ReceiveData: PVOID = core::ptr::null_mut();
    let ReceiveHeader: *mut KDVM_RECEIVE_HEADER;

    KdVmKdVmExchangeData(core::ptr::addr_of_mut!(ReceiveData), ReceiveDataSize);
    ReceiveHeader = ReceiveData as *mut KDVM_RECEIVE_HEADER;

    if *ReceiveDataSize < core::mem::size_of::<KDVM_RECEIVE_HEADER>() as ULONG
    {
        KDDBGPRINT!(b"KdVmSendReceive: received data too small: 0x%x\n\0".as_ptr() as PCHAR, *ReceiveDataSize);
        *ReceiveDataSize = 0;
        return core::ptr::null_mut();
    }

    if (*ReceiveHeader).Id != 0x2031
    {
        KDDBGPRINT!(b"KdVmSendReceive: got invalid Id: 0x%x\n\0".as_ptr() as PCHAR, (*ReceiveHeader).Id);
        *ReceiveDataSize = 0;
        return core::ptr::null_mut();
    }

    if RtlEqualMemory((*ReceiveHeader).Magic.as_ptr() as *const VOID, KdVmReplyMagic.as_ptr() as *const VOID, 9) == 0
    {
        KDDBGPRINT!(b"KdVmSendReceive: got invalid Magic: '%*s'\n\0".as_ptr() as PCHAR,
                   core::mem::size_of_val(&KdVmReplyMagic) as i32, (*ReceiveHeader).Magic.as_ptr());
        *ReceiveDataSize = 0;
        return core::ptr::null_mut();
    }

    *ReceiveDataSize = (*ReceiveDataSize).wrapping_sub(core::mem::size_of::<KDVM_RECEIVE_HEADER>() as ULONG);
    return ReceiveHeader.offset(1) as PVOID;
}

unsafe fn KdVmNegotiateProtocolVersions() -> NTSTATUS {

    let Version: ULONG = KDRPC_PROTOCOL_VERSION;
    let mut ReceivedSize: ULONG = 0;
    let ReceivedVersion: PULONG;

    KDDBGPRINT!(b"KdVmNegotiateProtocolVersions()\n\0".as_ptr() as PCHAR);

    KdVmPrepareBuffer();

    if KdVmAddCommandToBuffer(
        KDVM_CMD_VersionReport,
        core::ptr::addr_of!(Version) as PVOID,
        core::mem::size_of::<ULONG>(),
    ) == 0
    {
        KDDBGPRINT!(b"Failed to do VersionReport\n\0".as_ptr() as PCHAR);
        return STATUS_CONNECTION_REFUSED;
    }

    ReceivedVersion = KdVmSendReceive(core::ptr::addr_of_mut!(ReceivedSize)) as PULONG;
    if ReceivedSize != core::mem::size_of::<ULONG>() as ULONG {
        KDDBGPRINT!(
            b"Invalid size for VersionReport: %lx\n\0".as_ptr() as PCHAR,
            ReceivedSize,
        );
        return STATUS_CONNECTION_REFUSED;
    }

    if *ReceivedVersion != KDRPC_PROTOCOL_VERSION {
        KDDBGPRINT!(b"Invalid Version: %lx\n\0".as_ptr() as PCHAR, *ReceivedVersion);
        return STATUS_CONNECTION_REFUSED;
    }

    return STATUS_SUCCESS;
}

unsafe fn TestConnectionOnChannel() -> BOOLEAN {
    let mut TestBuffer: [UCHAR; KDRPC_TEST_BUFFER_SIZE as usize] = [0; KDRPC_TEST_BUFFER_SIZE as usize];
    let ReceivedBuffer: *mut UCHAR;
    let mut i: ULONG;
    let mut ReceivedSize: ULONG = 0;

    KdVmPrepareBuffer();

    i = 0;
    while i < core::mem::size_of_val(&TestBuffer) as ULONG {
        TestBuffer[i as usize] = i as UCHAR;
        i += 1;
    }

    if KdVmAddCommandToBuffer(KDVM_CMD_TestConnection, TestBuffer.as_ptr() as PVOID, core::mem::size_of_val(&TestBuffer)) == 0 {
        KDDBGPRINT!(b"Failed to do TestConnection\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    ReceivedBuffer = KdVmSendReceive(core::ptr::addr_of_mut!(ReceivedSize)) as *mut UCHAR;
    if ReceivedSize != core::mem::size_of_val(&TestBuffer) as ULONG {
        KDDBGPRINT!(b"Invalid size for TestConnection: %lx\n\0".as_ptr() as PCHAR, ReceivedSize);
        return 0u8;
    }

    i = 0;
    while i < core::mem::size_of_val(&TestBuffer) as ULONG {
        if *ReceivedBuffer.add(i as usize) != (i as UCHAR ^ 0x55) {
            KDDBGPRINT!(b"Wrong test data @ %lx, expected %x, got %x\n\0".as_ptr() as PCHAR,
                       i, i as UCHAR ^ 0x55, TestBuffer[i as usize]);
            return 0u8;
        }
        i += 1;
    }

    KDDBGPRINT!(b"TestConnectionOnChannel: success\n\0".as_ptr() as PCHAR);
    return 1u8;
}

unsafe fn KdVmTestConnectionWithHost() -> BOOLEAN {
    let mut i: ULONG;
    let mut j: ULONG;
    
    
    KDDBGPRINT!(b"KdVmTestConnectionWithHost()\n\0".as_ptr() as PCHAR);

    j = 0;
    while j < 2 {
        i = 0;
        while i < CONNECTION_TEST_ROUNDS / 2 {
            if TestConnectionOnChannel() == 0 {
                return 0u8;
            }
            i += 1;
        }
        j += 1;
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn KdD0Transition() -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdD3Transition() -> NTSTATUS {
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn KdSave(
    _SleepTransition: BOOLEAN
) -> NTSTATUS {
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn KdRestore(
    _SleepTransition: BOOLEAN
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdDebuggerInitialize0(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {

    let CommandLine: PCHAR;
    let mut PortString: PCHAR;
    let Status: NTSTATUS;

    if LoaderBlock != core::ptr::null_mut() {
        KdpDbgPrint = core::mem::transmute::<PVOID, PFNDBGPRNT>(
            (*LoaderBlock).u.I386.CommonDataArea,
        );
        KDDBGPRINT!(b"KdDebuggerInitialize0\n\0".as_ptr() as PCHAR);

        CommandLine = (*LoaderBlock).LoadOptions;

        _strupr(CommandLine);

        PortString = strstr(CommandLine, b"DEBUGPORT\0".as_ptr() as PCHAR);
        if PortString != core::ptr::null_mut() {
            PortString = PortString.add(strlen(b"DEBUGPORT\0".as_ptr() as PCHAR));

            while *PortString == b' ' as i8 {
                PortString = PortString.add(1);
            }
            /* Skip the equals sign, but never the terminator: a command line
             * ending in a bare "DEBUGPORT" would read past its end (upstream
             * 956d9f7973f) */
            if *PortString != 0 {
                PortString = PortString.add(1);
            }

            if strncmp(PortString, b"VBOX\0".as_ptr() as PCHAR, 4) != 0 {
                KDDBGPRINT!(b"Invalid debugport: '%s'\n\0".as_ptr() as PCHAR, CommandLine);
                return STATUS_INVALID_PARAMETER;
            }
        }
    }

    KdVmBufferPhysicalAddress = MmGetPhysicalAddress(KdVmDataBuffer.as_mut_ptr() as *mut core::ffi::c_void);
    KDDBGPRINT!(
        b"KdVmBufferPhysicalAddress = %llx\n\0".as_ptr() as PCHAR,
        (*core::ptr::addr_of!(KdVmBufferPhysicalAddress)).QuadPart,
    );

    Status = KdVmNegotiateProtocolVersions();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if KdVmTestConnectionWithHost() == 0 {
        return STATUS_CONNECTION_REFUSED;
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn KdDebuggerInitialize1(
    _LoaderBlock: PLOADER_PARAMETER_BLOCK
) -> NTSTATUS {
    
    KDDBGPRINT!(b"KdDebuggerInitialize1()\n\0".as_ptr() as *const i8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdSendPacket(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
    KdContext: PKD_CONTEXT,
) {
    let mut SendPktRequest: KDVM_SEND_PKT_REQUEST;
    let mut SendPktResult: *mut KDVM_SEND_PKT_RESULT;
    let mut ReceivedSize: ULONG = 0;

    KDDBGPRINT!(b"KdSendPacket(0x%lx, ...)\n\0".as_ptr() as PCHAR, PacketType);

    loop {
        SendPktRequest = core::mem::zeroed();

        SendPktRequest.PacketType = PacketType;
        SendPktRequest.Info.set_KdDebuggerNotPresent(KdDebuggerNotPresent);
        SendPktRequest.Info.set_KdDebuggerEnabledAvailable(1);
        SendPktRequest.Info.KdDebuggerEnabled = (*SharedUserData).KdDebuggerEnabled;

        if !MessageHeader.is_null() {
            SendPktRequest.MessageHeader.Length = (*MessageHeader).Length;
            SendPktRequest.MessageHeader.MaximumLength = (*MessageHeader).MaximumLength;
            SendPktRequest.HeaderSize = (*MessageHeader).Length as ULONG;
        }

        if !MessageData.is_null() {
            SendPktRequest.MessageData.Length = (*MessageData).Length;
            SendPktRequest.MessageData.MaximumLength = (*MessageData).MaximumLength;
            SendPktRequest.DataSize = (*MessageData).Length as ULONG;
        }

        if !KdContext.is_null() {
            RtlCopyMemory(
                core::ptr::addr_of_mut!(SendPktRequest.KdContext) as PVOID,
                KdContext as PVOID,
                core::mem::size_of_val(&SendPktRequest.KdContext),
            );
        }

        KdVmPrepareBuffer();

        if KdVmAddCommandToBuffer(
            KDVM_CMD_SendPacket,
            core::ptr::addr_of_mut!(SendPktRequest) as PVOID,
            core::mem::size_of_val(&SendPktRequest),
        ) == 0u8
        {
            KDDBGPRINT!(b"KdSendPacket: Failed to add SendPacket command\n\0".as_ptr() as PCHAR);
            return;
        }

        if !MessageHeader.is_null() {
            if KdVmAddToBuffer((*MessageHeader).Buffer as PVOID, (*MessageHeader).Length as ULONG) == 0u8 {
                KDDBGPRINT!(b"KdSendPacket: Failed to add MessageHeader\n\0".as_ptr() as PCHAR);
                return;
            }
        }

        if !MessageData.is_null() {
            if KdVmAddToBuffer((*MessageData).Buffer as PVOID, (*MessageData).Length as ULONG) == 0u8 {
                KDDBGPRINT!(b"KdSendPacket: Failed to add MessageData\n\0".as_ptr() as PCHAR);
                return;
            }
        }

        SendPktResult = KdVmSendReceive(core::ptr::addr_of_mut!(ReceivedSize)) as *mut KDVM_SEND_PKT_RESULT;
        if ReceivedSize != core::mem::size_of::<KDVM_SEND_PKT_RESULT>() as ULONG {
            KDDBGPRINT!(
                b"KdSendPacket: Invalid size for SendPktResult: %lx\n\0".as_ptr() as PCHAR,
                ReceivedSize,
            );
            return;
        }

        if !KdContext.is_null() {
            RtlCopyMemory(
                KdContext as PVOID,
                core::ptr::addr_of_mut!((*SendPktResult).KdContext) as PVOID,
                core::mem::size_of_val(&(*SendPktResult).KdContext),
            );
        }

        KdDebuggerNotPresent = (*SendPktResult).Info.KdDebuggerNotPresent();
        if (*SendPktResult).Info.KdDebuggerEnabledAvailable() != 0 {
            (*SharedUserData).KdDebuggerEnabled = if (*SendPktResult).Info.KdDebuggerEnabled != 0 { 1 } else { 0 };
        }

        if (*SendPktResult).Info.RetryKdSendPacket() != 0 {
            KDDBGPRINT!(b"KdSendPacket: RetryKdSendPacket!\n\0".as_ptr() as PCHAR);
        }

        if (*SendPktResult).Info.RetryKdSendPacket() == 0 {
            break;
        }
    }

    KDDBGPRINT!(b"KdSendPacket: Success!\n\0".as_ptr() as PCHAR);
}

#[no_mangle]
pub unsafe extern "system" fn KdReceivePacket(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
    DataLength: PULONG,
    KdContext: PKD_CONTEXT,
) -> KDP_STATUS {

    let mut RecvPktRequest: KDVM_RECV_PKT_REQUEST = core::mem::zeroed();
    let RecvPktResult: *mut KDVM_RECV_PKT_RESULT;
    let mut ReceivedSize: ULONG = 0;
    let ExpectedSize: ULONG;
    let mut Buffer: PUCHAR;

    KDDBGPRINT!(b"KdReceivePacket(0x%lx, ...)\n\0".as_ptr() as PCHAR, PacketType);

    KdVmPrepareBuffer();

    RtlZeroMemory(
        core::ptr::addr_of_mut!(RecvPktRequest) as PVOID,
        core::mem::size_of::<KDVM_RECV_PKT_REQUEST>(),
    );

    RecvPktRequest.PacketType = PacketType;
    RecvPktRequest.Info.set_KdDebuggerNotPresent(KdDebuggerNotPresent);
    RecvPktRequest.Info.set_KdDebuggerEnabledAvailable(1);
    RecvPktRequest.Info.KdDebuggerEnabled = (*SharedUserData).KdDebuggerEnabled;

    if !MessageHeader.is_null() {
        RecvPktRequest.MessageHeader.Length = (*MessageHeader).Length;
        RecvPktRequest.MessageHeader.MaximumLength = (*MessageHeader).MaximumLength;
    }

    if !MessageData.is_null() {
        RecvPktRequest.MessageData.Length = (*MessageData).Length;
        RecvPktRequest.MessageData.MaximumLength = (*MessageData).MaximumLength;
    }

    if !KdContext.is_null() {
        RtlCopyMemory(
            core::ptr::addr_of_mut!(RecvPktRequest.KdContext) as PVOID,
            KdContext as PVOID,
            core::mem::size_of_val(&RecvPktRequest.KdContext),
        );
    }

    if KdVmAddCommandToBuffer(
        KDVM_CMD_ReceivePacket,
        core::ptr::addr_of_mut!(RecvPktRequest) as PVOID,
        core::mem::size_of::<KDVM_RECV_PKT_REQUEST>(),
    ) == 0
    {
        KDDBGPRINT!(b"KdReceivePacket: Failed to add SendPacket command\n\0".as_ptr() as PCHAR);
        return KDP_PACKET_RESEND;
    }

    RecvPktResult = KdVmSendReceive(core::ptr::addr_of_mut!(ReceivedSize)) as *mut KDVM_RECV_PKT_RESULT;
    if ReceivedSize < core::mem::size_of::<KDVM_RECV_PKT_RESULT>() as ULONG {
        KDDBGPRINT!(
            b"KdReceivePacket: Invalid size for RecvPktResult: %lx\n\0".as_ptr() as PCHAR,
            ReceivedSize,
        );
        return KDP_PACKET_RESEND;
    }

    ExpectedSize = core::mem::size_of::<KDVM_RECV_PKT_RESULT>() as ULONG
        + (*RecvPktResult).HeaderSize
        + (*RecvPktResult).DataSize;
    if ReceivedSize != ExpectedSize {
        KDDBGPRINT!(
            b"KdReceivePacket: Invalid size for RecvPktResult: %lu, expected %lu\n\0".as_ptr() as PCHAR,
            ReceivedSize,
            ExpectedSize,
        );
        return KDP_PACKET_RESEND;
    }

    if !KdContext.is_null() {
        RtlCopyMemory(
            KdContext as PVOID,
            core::ptr::addr_of!((*RecvPktResult).KdContext) as PVOID,
            core::mem::size_of_val(&(*RecvPktResult).KdContext),
        );
    }

    Buffer = RecvPktResult.offset(1) as PUCHAR;
    if !MessageHeader.is_null() {
        (*MessageHeader).Length = (*RecvPktResult).MessageHeader.Length;
        if !(*MessageHeader).Buffer.is_null()
            && (*MessageHeader).MaximumLength >= (*RecvPktResult).HeaderSize as USHORT
        {
            RtlCopyMemory(
                (*MessageHeader).Buffer as PVOID,
                Buffer as PVOID,
                (*RecvPktResult).HeaderSize as SIZE_T,
            );
        } else {
            KDDBGPRINT!(b"MessageHeader not good\n\0".as_ptr() as PCHAR);
        }
    }

    Buffer = Buffer.offset((*RecvPktResult).HeaderSize as isize);
    if !MessageData.is_null() {
        (*MessageData).Length = (*RecvPktResult).MessageData.Length;
        if !(*MessageData).Buffer.is_null()
            && (*MessageData).MaximumLength >= (*RecvPktResult).DataSize as USHORT
        {
            RtlCopyMemory(
                (*MessageData).Buffer as PVOID,
                Buffer as PVOID,
                (*RecvPktResult).DataSize as SIZE_T,
            );
        } else {
            KDDBGPRINT!(b"MessageData not good\n\0".as_ptr() as PCHAR);
        }
    }

    if !DataLength.is_null() {
        *DataLength = (*RecvPktResult).FullSize;
    }

    KDDBGPRINT!(
        b"KdReceivePacket: returning status %u\n\0".as_ptr() as PCHAR,
        (*RecvPktResult).KdStatus,
    );
    return (*RecvPktResult).KdStatus;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kdvm/kdvm.c
//  * PURPOSE:         VM independent function for kdvbox/kd
//  * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// #include "kdvm.h"
//
// static CHAR KdVmCmdMagic[] = "~kdVMvA ";
// static CHAR KdVmReplyMagic[] = "++kdVMvA ";
// static const UCHAR KDVM_CMD_TestConnection = 't';
// static const UCHAR KDVM_CMD_ReceivePacket = 'r';
// static const UCHAR KDVM_CMD_SendPacket = 's';
// static const UCHAR KDVM_CMD_VersionReport = 'v';
//
// UCHAR KdVmDataBuffer[KDVM_BUFFER_SIZE];
// PHYSICAL_ADDRESS KdVmBufferPhysicalAddress;
// ULONG KdVmBufferPos;
//
// PFNDBGPRNT KdpDbgPrint;
//
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// static
// VOID
// KdVmDbgDumpRow(
//      PUCHAR Buffer,
//      ULONG Size)
// {
//     ULONG i;
//     for (i = 0;i < Size; i++)
//     {
//         KdpDbgPrint("%02x ", Buffer[i]);
//     }
//     KdpDbgPrint("\n");
// }
//
// VOID
// NTAPI
// KdVmDbgDumpBuffer(
//      PVOID Buffer,
//      ULONG Size)
// {
//     PUCHAR CurrentRow;
//     ULONG i;
//
//     CurrentRow = Buffer;
//     for (i = 0; i < (Size / 16); i++)
//     {
//         KdVmDbgDumpRow(CurrentRow, 16);
//         CurrentRow += 16;
//     }
//     KdVmDbgDumpRow(CurrentRow, (Size % 16));
// }
//
// static
// BOOLEAN
// KdVmAddToBuffer(
//      PVOID Data,
//      ULONG DataSize)
// {
//     if (((KdVmBufferPos + DataSize) > KDVM_BUFFER_SIZE) ||
//         ((KdVmBufferPos + DataSize) < KdVmBufferPos))
//     {
//         KDDBGPRINT("KdVmAddToBuffer: Buffer overflow! Need %lu, remaining: %lu\n",
//                    DataSize, KDVM_BUFFER_SIZE - KdVmBufferPos);
//         return FALSE;
//     }
//
//     RtlCopyMemory(&KdVmDataBuffer[KdVmBufferPos], Data, DataSize);
//     KdVmBufferPos += DataSize;
//     return TRUE;
// }
//
// static
// BOOLEAN
// KdVmAddCommandToBuffer(
//      UCHAR Command,
//      PVOID Buffer,
//      SIZE_T BufferSize)
// {
//     KDVM_CMD_HEADER Header;
//
//     RtlCopyMemory(&Header.Magic, KdVmCmdMagic, sizeof(Header.Magic));
//     Header.Command = Command;
//
//     if (!KdVmAddToBuffer(&Header, sizeof(Header)))
//         return FALSE;
//
//     if (!KdVmAddToBuffer(Buffer, BufferSize))
//         return FALSE;
//
//     return TRUE;
// }
//
// static
// PVOID
// KdVmSendReceive(
//      PULONG ReceiveDataSize)
// {
//     PVOID ReceiveData;
//     PKDVM_RECEIVE_HEADER ReceiveHeader;
//
//     KdVmKdVmExchangeData(&ReceiveData, ReceiveDataSize);
//     ReceiveHeader = ReceiveData;
//
//     if (*ReceiveDataSize < sizeof(*ReceiveHeader))
//     {
//         KDDBGPRINT("KdVmSendReceive: received data too small: 0x%x\n", *ReceiveDataSize);
//         *ReceiveDataSize = 0;
//         return NULL;
//     }
//
//     if (ReceiveHeader->Id != 0x2031 /* '01' */)
//     {
//         KDDBGPRINT("KdVmSendReceive: got invalid Id: 0x%x\n", ReceiveHeader->Id);
//         *ReceiveDataSize = 0;
//         return NULL;
//     }
//
//     if (!RtlEqualMemory(ReceiveHeader->Magic, KdVmReplyMagic, 9))
//     {
//         KDDBGPRINT("KdVmSendReceive: got invalid Magic: '%*s'\n",
//                    sizeof(KdVmReplyMagic), ReceiveHeader->Magic);
//         *ReceiveDataSize = 0;
//         return NULL;
//     }
//
//     *ReceiveDataSize -= sizeof(*ReceiveHeader);
//     return (PVOID)(ReceiveHeader + 1);
// }
//
// static
// NTSTATUS
// KdVmNegotiateProtocolVersions(VOID)
// {
//     ULONG Version = KDRPC_PROTOCOL_VERSION;
//     ULONG ReceivedSize;
//     PULONG ReceivedVersion;
//     KDDBGPRINT("KdVmNegotiateProtocolVersions()\n");
//
//     /* Prepare the buffer */
//     KdVmPrepareBuffer();
//
//     if (!KdVmAddCommandToBuffer(KDVM_CMD_VersionReport, &Version, sizeof(Version)))
//     {
//         KDDBGPRINT("Failed to do VersionReport\n");
//         return STATUS_CONNECTION_REFUSED;
//     }
//
//     ReceivedVersion = KdVmSendReceive(&ReceivedSize);
//     if (ReceivedSize != sizeof(ULONG))
//     {
//         KDDBGPRINT("Invalid size for VersionReport: %lx\n", ReceivedSize);
//         return STATUS_CONNECTION_REFUSED;
//     }
//
//     if (*ReceivedVersion != KDRPC_PROTOCOL_VERSION)
//     {
//         KDDBGPRINT("Invalid Version: %lx\n", *ReceivedVersion);
//         return STATUS_CONNECTION_REFUSED; //STATUS_PROTOCOL_NOT_SUPPORTED;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// static
// BOOLEAN
// TestConnectionOnChannel(VOID)
// {
//     UCHAR TestBuffer[KDRPC_TEST_BUFFER_SIZE];
//     PUCHAR ReceivedBuffer;
//     ULONG i, ReceivedSize;
//
//     /* Prepare the buffer */
//     KdVmPrepareBuffer();
//
//     for (i = 0; i < sizeof(TestBuffer); i++)
//         TestBuffer[i] = (UCHAR)i;
//
//     if (!KdVmAddCommandToBuffer(KDVM_CMD_TestConnection, TestBuffer, sizeof(TestBuffer)))
//     {
//         KDDBGPRINT("Failed to do TestConnection\n");
//         return FALSE;
//     }
//
//     ReceivedBuffer = KdVmSendReceive(&ReceivedSize);
//     if (ReceivedSize != sizeof(TestBuffer))
//     {
//         KDDBGPRINT("Invalid size for TestConnection: %lx\n", ReceivedSize);
//         return FALSE;
//     }
//
//     for (i = 0; i < sizeof(TestBuffer); i++)
//     {
//         if (ReceivedBuffer[i] != (UCHAR)(i ^ 0x55))
//         {
//             KDDBGPRINT("Wrong test data @ %lx, expected %x, got %x\n",
//                        i, (UCHAR)(i ^ 0x55), TestBuffer[i]);
//             return FALSE;
//         }
//     }
//
//     KDDBGPRINT("TestConnectionOnChannel: success\n");
//     return TRUE;
// }
//
// static
// BOOLEAN
// KdVmTestConnectionWithHost(VOID)
// {
//     ULONG i, j;
//     KDDBGPRINT("KdVmTestConnectionWithHost()\n");
//
//     for (j = 0; j < 2; j++)
//     {
//         //VMWareRPC::OpenChannel
//         for (i = 0; i < CONNECTION_TEST_ROUNDS / 2; i++)
//         {
//             if (!TestConnectionOnChannel())
//             {
//                 return FALSE;
//             }
//         }
//     }
//
//     return TRUE;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// NTSTATUS
// NTAPI
// KdD0Transition(VOID)
// {
//     /* Nothing to do */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// KdD3Transition(VOID)
// {
//     /* Nothing to do */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// KdSave(
//      BOOLEAN SleepTransition)
// {
//     /* Nothing to do */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// KdRestore(
//      BOOLEAN SleepTransition)
// {
//     /* Nothing to do */
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * \name KdDebuggerInitialize0
//  * \brief Phase 0 initialization.
//  * \param [opt] LoaderBlock Pointer to the Loader parameter block. Can be NULL.
//  * \return Status
//  */
// NTSTATUS
// NTAPI
// KdDebuggerInitialize0(
//      PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PCHAR CommandLine, PortString;
//     NTSTATUS Status;
//
//     /* Check if we have a LoaderBlock */
//     if (LoaderBlock != NULL)
//     {
//         /* HACK */
//         KdpDbgPrint = LoaderBlock->u.I386.CommonDataArea;
//         KDDBGPRINT("KdDebuggerInitialize0\n");
//
//         /* Get the Command Line */
//         CommandLine = LoaderBlock->LoadOptions;
//
//         /* Upcase it */
//         _strupr(CommandLine);
//
//         /* Check if we got the /DEBUGPORT parameter */
//         PortString = strstr(CommandLine, "DEBUGPORT");
//         if (PortString)
//         {
//             /* Move past the actual string, to reach the port*/
//             PortString += strlen("DEBUGPORT");
//
//             /* Now get past any spaces and skip the equal sign */
//             while (*PortString == ' ') PortString++;
//             PortString++;
//
//             /* Do we have a serial port? */
//             if (strncmp(PortString, "VBOX", 4) != 0)
//             {
//                 KDDBGPRINT("Invalid debugport: '%s'\n", CommandLine);
//                 return STATUS_INVALID_PARAMETER;
//             }
//         }
//     }
//
//     /* Get the physical address of the data buffer */
//     KdVmBufferPhysicalAddress = MmGetPhysicalAddress(KdVmDataBuffer);
//     KDDBGPRINT("KdVmBufferPhysicalAddress = %llx\n", KdVmBufferPhysicalAddress.QuadPart);
//
//     Status = KdVmNegotiateProtocolVersions();
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (!KdVmTestConnectionWithHost())
//         return STATUS_CONNECTION_REFUSED;
//
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * \name KdDebuggerInitialize1
//  * \brief Phase 1 initialization.
//  * \param [opt] LoaderBlock Pointer to the Loader parameter block. Can be NULL.
//  * \return Status
//  */
// NTSTATUS
// NTAPI
// KdDebuggerInitialize1(
//      PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     /* Nothing to do */
//     KDDBGPRINT("KdDebuggerInitialize1()\n");
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// KdSendPacket(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData,
//      PKD_CONTEXT KdContext)
// {
//     KDVM_SEND_PKT_REQUEST SendPktRequest;
//     PKDVM_SEND_PKT_RESULT SendPktResult;
//     ULONG ReceivedSize;
//     KDDBGPRINT("KdSendPacket(0x%lx, ...)\n", PacketType);
//
//     do
//     {
//
//         RtlZeroMemory(&SendPktRequest, sizeof(SendPktRequest));
//
//         SendPktRequest.PacketType = PacketType;
//         SendPktRequest.Info.KdDebuggerNotPresent = KD_DEBUGGER_NOT_PRESENT;
//         SendPktRequest.Info.KdDebuggerEnabledAvailable = 1;
//         SendPktRequest.Info.KdDebuggerEnabled = SharedUserData->KdDebuggerEnabled;
//
//         if (MessageHeader != NULL)
//         {
//             SendPktRequest.MessageHeader.Length = MessageHeader->Length;
//             SendPktRequest.MessageHeader.MaximumLength = MessageHeader->MaximumLength;
//             SendPktRequest.HeaderSize = MessageHeader->Length;
//         }
//
//         if (MessageData != NULL)
//         {
//             SendPktRequest.MessageData.Length = MessageData->Length;
//             SendPktRequest.MessageData.MaximumLength = MessageData->MaximumLength;
//             SendPktRequest.DataSize = MessageData->Length;
//         }
//
//         if (KdContext != NULL)
//         {
//             RtlCopyMemory(&SendPktRequest.KdContext,
//                           KdContext,
//                           sizeof(SendPktRequest.KdContext));
//         }
//
//
//         /* Prepare the buffer */
//         KdVmPrepareBuffer();
//
//         if (!KdVmAddCommandToBuffer(KDVM_CMD_SendPacket, &SendPktRequest, sizeof(SendPktRequest)))
//         {
//             KDDBGPRINT("KdSendPacket: Failed to add SendPacket command\n");
//             return;
//         }
//
//         if (MessageHeader != NULL)
//         {
//             if (!KdVmAddToBuffer(MessageHeader->Buffer, MessageHeader->Length))
//             {
//                 KDDBGPRINT("KdSendPacket: Failed to add MessageHeader\n");
//                 return;
//             }
//         }
//
//         if (MessageData != NULL)
//         {
//             if (!KdVmAddToBuffer(MessageData->Buffer, MessageData->Length))
//             {
//                 KDDBGPRINT("KdSendPacket: Failed to add MessageData\n");
//                 return;
//             }
//         }
//
//         SendPktResult = KdVmSendReceive(&ReceivedSize);
//         if (ReceivedSize != sizeof(*SendPktResult))
//         {
//             KDDBGPRINT("KdSendPacket: Invalid size for SendPktResult: %lx\n", ReceivedSize);
//             return;
//         }
//
//         if (KdContext != NULL)
//         {
//             RtlCopyMemory(KdContext,
//                           &SendPktResult->KdContext,
//                           sizeof(SendPktResult->KdContext));
//         }
//
//         KD_DEBUGGER_NOT_PRESENT = SendPktResult->Info.KdDebuggerNotPresent;
//         if (SendPktResult->Info.KdDebuggerEnabledAvailable)
//             SharedUserData->KdDebuggerEnabled = SendPktResult->Info.KdDebuggerEnabled != 0;
//
//         if (SendPktResult->Info.RetryKdSendPacket)
//         {
//             KDDBGPRINT("KdSendPacket: RetryKdSendPacket!\n");
//         }
//
//     } while (SendPktResult->Info.RetryKdSendPacket);
//
//     KDDBGPRINT("KdSendPacket: Success!\n");
// }
//
//
// KDP_STATUS
// NTAPI
// KdReceivePacket(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData,
//      PULONG DataLength,
//      PKD_CONTEXT KdContext)
// {
//     KDVM_RECV_PKT_REQUEST RecvPktRequest;
//     PKDVM_RECV_PKT_RESULT RecvPktResult;
//     ULONG ReceivedSize, ExpectedSize;
//     PUCHAR Buffer;
//     KDDBGPRINT("KdReceivePacket(0x%lx, ...)\n", PacketType);
//
//     /* Prepare the buffer */
//     KdVmPrepareBuffer();
//
//     RtlZeroMemory(&RecvPktRequest, sizeof(RecvPktRequest));
//
//     RecvPktRequest.PacketType = PacketType;
//     RecvPktRequest.Info.KdDebuggerNotPresent = KD_DEBUGGER_NOT_PRESENT;
//     RecvPktRequest.Info.KdDebuggerEnabledAvailable = 1;
//     RecvPktRequest.Info.KdDebuggerEnabled = SharedUserData->KdDebuggerEnabled;
//
//     if (MessageHeader != NULL)
//     {
//         RecvPktRequest.MessageHeader.Length = MessageHeader->Length;
//         RecvPktRequest.MessageHeader.MaximumLength = MessageHeader->MaximumLength;
//     }
//
//     if (MessageData != NULL)
//     {
//         RecvPktRequest.MessageData.Length = MessageData->Length;
//         RecvPktRequest.MessageData.MaximumLength = MessageData->MaximumLength;
//     }
//
//     if (KdContext != NULL)
//     {
//         RtlCopyMemory(&RecvPktRequest.KdContext,
//                       KdContext,
//                       sizeof(RecvPktRequest.KdContext));
//     }
//
//     if (!KdVmAddCommandToBuffer(KDVM_CMD_ReceivePacket, &RecvPktRequest, sizeof(RecvPktRequest)))
//     {
//         KDDBGPRINT("KdReceivePacket: Failed to add SendPacket command\n");
//         return KDP_PACKET_RESEND;
//     }
//
//     RecvPktResult = KdVmSendReceive(&ReceivedSize);
//     if (ReceivedSize < sizeof(*RecvPktResult))
//     {
//         KDDBGPRINT("KdReceivePacket: Invalid size for RecvPktResult: %lx\n", ReceivedSize);
//         return KDP_PACKET_RESEND;
//     }
//
//     ExpectedSize = sizeof(*RecvPktResult) +
//                    RecvPktResult->HeaderSize +
//                    RecvPktResult->DataSize;
//     if (ReceivedSize != ExpectedSize)
//     {
//         KDDBGPRINT("KdReceivePacket: Invalid size for RecvPktResult: %lu, expected %lu\n",
//                    ReceivedSize, ExpectedSize);
//         return KDP_PACKET_RESEND;
//     }
//
//     if (KdContext != NULL)
//     {
//         RtlCopyMemory(KdContext,
//                       &RecvPktResult->KdContext,
//                       sizeof(RecvPktResult->KdContext));
//     }
//
//     Buffer = (PUCHAR)(RecvPktResult + 1);
//     if (MessageHeader != NULL)
//     {
//         MessageHeader->Length = RecvPktResult->MessageHeader.Length;
//         if ((MessageHeader->Buffer != NULL) &&
//             (MessageHeader->MaximumLength >= RecvPktResult->HeaderSize))
//         {
//             RtlCopyMemory(MessageHeader->Buffer,
//                           Buffer,
//                           RecvPktResult->HeaderSize);
//         }
//         else
//         {
//             KDDBGPRINT("MessageHeader not good\n");
//         }
//     }
//
//     Buffer += RecvPktResult->HeaderSize;
//     if (MessageData != NULL)
//     {
//         MessageData->Length = RecvPktResult->MessageData.Length;
//         if ((MessageData->Buffer != NULL) &&
//             (MessageData->MaximumLength >= RecvPktResult->DataSize))
//         {
//             RtlCopyMemory(MessageData->Buffer,
//                           Buffer,
//                           RecvPktResult->DataSize);
//         }
//         else
//         {
//             KDDBGPRINT("MessageData not good\n");
//         }
//     }
//
//     if (DataLength != NULL)
//         *DataLength = RecvPktResult->FullSize;
//
//     KDDBGPRINT("KdReceivePacket: returning status %u\n", RecvPktResult->KdStatus);
//     return RecvPktResult->KdStatus;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
