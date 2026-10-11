use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const SEED_BUFFER_SIZE: usize = 6;

unsafe extern "C" fn getMacAddress(MacAddress: *mut UCHAR) -> BOOLEAN {
    // UNIMPLEMENTED;
    (FALSE as u8)
}

unsafe extern "C" fn CookupNodeId(NodeId: *mut UCHAR) {
    let mut ComputerName: [CHAR; (MAX_COMPUTERNAME_LENGTH + 1) as _] = [0; (MAX_COMPUTERNAME_LENGTH + 1) as _];
    let mut CurrentChar: *mut CHAR = core::ptr::null_mut();
    let mut Size: DWORD = 0;
    let mut PerformanceCount: LARGE_INTEGER = core::mem::zeroed();
    let mut NodeBegin: PDWORD = core::ptr::null_mut();
    let mut NodeMiddle: PDWORD = core::ptr::null_mut();
    let mut dwValue: DWORD = 0;
    let mut MemoryStatus: MEMORYSTATUS = core::mem::zeroed();
    let mut Luid: LUID = core::mem::zeroed();
    let mut szSystem: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FreeBytesToCaller: ULARGE_INTEGER = core::mem::zeroed();
    let mut TotalBytes: ULARGE_INTEGER = core::mem::zeroed();
    let mut TotalFreeBytes: ULARGE_INTEGER = core::mem::zeroed();

    /* Initialize node id */
    memset(
        NodeId as *mut core::ffi::c_void,
        0x11,
        SEED_BUFFER_SIZE * core::mem::size_of::<UCHAR>(),
    );

    /* Randomize it with computer name */
    Size = (MAX_COMPUTERNAME_LENGTH + 1) as DWORD;
    if GetComputerNameA(
        core::ptr::addr_of_mut!(ComputerName) as *mut CHAR,
        core::ptr::addr_of_mut!(Size),
    ) != 0
    {
        Size = 0 as DWORD;
        CurrentChar = (core::ptr::addr_of_mut!(ComputerName) as *mut CHAR)
            .offset(0 as isize);
        while *CurrentChar != ANSI_NULL {
            *NodeId.add(Size as usize) ^= *CurrentChar as UCHAR;
            CurrentChar = CurrentChar.offset(1);

            /* Don't overflow our NodeId */
            Size = Size.wrapping_add(1);
            if Size >= SEED_BUFFER_SIZE as DWORD {
                Size = 0 as DWORD;
            }
        }
    }

    /* Now, we'll work directly on DWORD values */
    // NodeId is a byte array: these DWORDs are unaligned (C tolerates it; a Rust `*p` may not).
    NodeBegin = NodeId.offset(0 as isize) as *mut DWORD;
    NodeMiddle = NodeId.offset(2 as isize) as *mut DWORD;

    /* Randomize with performance counter */
    if QueryPerformanceCounter(core::ptr::addr_of_mut!(PerformanceCount)) != 0 {
        NodeMiddle.write_unaligned(NodeMiddle.read_unaligned() ^ (PerformanceCount.u.HighPart as DWORD) ^ PerformanceCount.u.LowPart);

        dwValue = (PerformanceCount.u.HighPart as DWORD) ^ PerformanceCount.u.LowPart ^ NodeBegin.read_unaligned();
    } else {
        dwValue = NodeBegin.read_unaligned();
    }

    NodeBegin.write_unaligned(NodeBegin.read_unaligned() ^ dwValue);
    NodeMiddle.write_unaligned(NodeMiddle.read_unaligned() ^ NodeBegin.read_unaligned());

    /* Then, with memory status */
    MemoryStatus.dwLength = core::mem::size_of::<MEMORYSTATUS>() as DWORD;
    GlobalMemoryStatus(core::ptr::addr_of_mut!(MemoryStatus));

    NodeBegin.write_unaligned(NodeBegin.read_unaligned() ^ MemoryStatus.dwMemoryLoad);
    NodeMiddle.write_unaligned(NodeMiddle.read_unaligned() ^ (MemoryStatus.dwTotalPhys) as u32);
    NodeBegin.write_unaligned(NodeBegin.read_unaligned() ^ (MemoryStatus.dwTotalPageFile) as u32 ^ (MemoryStatus.dwAvailPhys) as u32);
    NodeMiddle.write_unaligned(NodeMiddle.read_unaligned() ^ (MemoryStatus.dwTotalVirtual) as u32 ^ (MemoryStatus.dwAvailPageFile) as u32);
    NodeBegin.write_unaligned(NodeBegin.read_unaligned() ^ (MemoryStatus.dwAvailVirtual) as u32);

    /* With a LUID */
    if AllocateLocallyUniqueId(core::ptr::addr_of_mut!(Luid)) != 0 {
        NodeBegin.write_unaligned(NodeBegin.read_unaligned() ^ Luid.LowPart);
        NodeMiddle.write_unaligned(NodeMiddle.read_unaligned() ^ (Luid.HighPart as DWORD));
    }

    /* Get system directory */
    GetSystemDirectoryW(
        core::ptr::addr_of_mut!(szSystem) as *mut WCHAR,
        ARRAYSIZE!(szSystem),
    );

    /* And finally with free disk space */
    if GetDiskFreeSpaceExW(
        core::ptr::addr_of_mut!(szSystem) as *mut WCHAR,
        core::ptr::addr_of_mut!(FreeBytesToCaller),
        core::ptr::addr_of_mut!(TotalBytes),
        core::ptr::addr_of_mut!(TotalFreeBytes),
    ) != 0
    {
        NodeMiddle.write_unaligned(NodeMiddle.read_unaligned() ^ (FreeBytesToCaller.u.LowPart ^ TotalFreeBytes.u.HighPart));
        NodeMiddle.write_unaligned(NodeMiddle.read_unaligned() ^ (FreeBytesToCaller.u.HighPart ^ TotalFreeBytes.u.LowPart));
        NodeBegin.write_unaligned(NodeBegin.read_unaligned() ^ (TotalBytes.u.LowPart ^ TotalFreeBytes.u.LowPart));
        NodeBegin.write_unaligned(NodeBegin.read_unaligned() ^ (TotalBytes.u.HighPart ^ TotalFreeBytes.u.HighPart));
    }

    /*
     * Because it was locally generated, force the seed to be local
     * setting this, will trigger the check for validness in the kernel
     * and make the seed local
     */
    *NodeId.add(0 as usize) |= 0x80 as UCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn DealWithDeviceEvent() {
    let mut Status: NTSTATUS;
    let mut UuidSeed: [UCHAR; SEED_BUFFER_SIZE] = [0; SEED_BUFFER_SIZE];

    /* First, try to get a multicast MAC address */
    if getMacAddress(core::ptr::addr_of_mut!(UuidSeed) as *mut UCHAR) == 0 {
        DPRINT1!("Failed finding a proper MAC address, will generate seed\n");
        CookupNodeId(core::ptr::addr_of_mut!(UuidSeed) as *mut UCHAR);
    }

    /* Seed our UUID generator */
    Status = NtSetUuidSeed(core::ptr::addr_of_mut!(UuidSeed) as PUCHAR);
    if !NT_SUCCESS!(Status) {
        DPRINT1!("NtSetUuidSeed failed with status: %lx\n", Status);
    }
}
