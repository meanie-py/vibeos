//! Rust translation of `ntoskrnl/tests/dispatch.c` (commit 240c5e0b context).
//!
//! Serial command loop. Pytest sends commands; kernel runs tests and reports
//! results. See `dispatch.c` original header for the wire protocol.

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{KeStallExecutionProcessor, CPPORT};
unsafe extern "system" {
    fn KdPortGetByteEx(PortInformation: *mut CPPORT, Byte: *mut u8) -> BOOLEAN;
    static mut SerialPortInfo: CPPORT;
}

type KernelTestFunc = unsafe extern "system" fn() -> BOOLEAN;

unsafe extern "system" {
    fn NtTestMmPool() -> BOOLEAN;
    fn NtTestObHandle() -> BOOLEAN;
    fn NtTestKeEvent() -> BOOLEAN;
    fn NtTestDpc() -> BOOLEAN;
    fn NtTestTimer() -> BOOLEAN;
    fn NtTestVmem() -> BOOLEAN;
    fn NtTestRegistry() -> BOOLEAN;
    fn NtTestWorkItem() -> BOOLEAN;
    fn NtTestIrql() -> BOOLEAN;
    fn NtTestThread() -> BOOLEAN;
    fn NtTestMutex() -> BOOLEAN;
    fn NtTestSemaphore() -> BOOLEAN;
    fn NtTestMultiWait() -> BOOLEAN;
    fn NtTestObRef() -> BOOLEAN;
    fn NtTestRtlStr() -> BOOLEAN;
    fn NtTestSList() -> BOOLEAN;
    fn NtTestMdl() -> BOOLEAN;
    fn NtTestFileIo() -> BOOLEAN;
    fn NtTestLookaside() -> BOOLEAN;
    fn NtTestFastMutex() -> BOOLEAN;
    fn NtTestBitmap() -> BOOLEAN;
    fn NtTestCapture() -> BOOLEAN;
    fn NtTestMemOps() -> BOOLEAN;
    fn NtTestSysTime() -> BOOLEAN;
    fn NtTestProcInfo() -> BOOLEAN;
    fn NtTestObjDir() -> BOOLEAN;
    fn NtTestSymLink() -> BOOLEAN;
    fn NtTestInterlockedList() -> BOOLEAN;
}

struct TestEntry {
    name: &'static [u8],   // NUL-terminated
    func: KernelTestFunc,
}

static TEST_TABLE: &[TestEntry] = &[
    TestEntry { name: b"MmPool\0",     func: NtTestMmPool },
    TestEntry { name: b"ObHandle\0",   func: NtTestObHandle },
    TestEntry { name: b"KeEvent\0",    func: NtTestKeEvent },
    TestEntry { name: b"Dpc\0",        func: NtTestDpc },
    TestEntry { name: b"Timer\0",      func: NtTestTimer },
    TestEntry { name: b"Vmem\0",       func: NtTestVmem },
    TestEntry { name: b"Registry\0",   func: NtTestRegistry },
    TestEntry { name: b"WorkItem\0",   func: NtTestWorkItem },
    TestEntry { name: b"Irql\0",       func: NtTestIrql },
    TestEntry { name: b"Thread\0",     func: NtTestThread },
    TestEntry { name: b"Mutex\0",      func: NtTestMutex },
    TestEntry { name: b"Semaphore\0",  func: NtTestSemaphore },
    TestEntry { name: b"MultiWait\0",  func: NtTestMultiWait },
    TestEntry { name: b"ObRef\0",      func: NtTestObRef },
    TestEntry { name: b"RtlStr\0",     func: NtTestRtlStr },
    TestEntry { name: b"SList\0",      func: NtTestSList },
    TestEntry { name: b"Mdl\0",        func: NtTestMdl },
    TestEntry { name: b"FileIo\0",     func: NtTestFileIo },
    TestEntry { name: b"Lookaside\0",  func: NtTestLookaside },
    TestEntry { name: b"FastMutex\0",  func: NtTestFastMutex },
    TestEntry { name: b"Bitmap\0",     func: NtTestBitmap },
    TestEntry { name: b"Capture\0",    func: NtTestCapture },
    TestEntry { name: b"MemOps\0",     func: NtTestMemOps },
    TestEntry { name: b"SysTime\0",    func: NtTestSysTime },
    TestEntry { name: b"ProcInfo\0",   func: NtTestProcInfo },
    TestEntry { name: b"ObjDir\0",     func: NtTestObjDir },
    TestEntry { name: b"SymLink\0",    func: NtTestSymLink },
    TestEntry { name: b"InterlockedList\0", func: NtTestInterlockedList },
];

unsafe fn read_byte(byte: *mut u8) -> bool {
    KdPortGetByteEx(core::ptr::addr_of_mut!(SerialPortInfo), byte) != 0
}

/// Read a line into `buffer` (up to `buffer.len() - 1` chars). Blocks until \n.
/// Returns length excluding \n. The result is NUL-terminated.
unsafe fn read_line(buffer: &mut [u8]) -> usize {
    let max = buffer.len();
    let mut pos = 0usize;
    let mut byte: u8 = 0;
    while pos < max - 1 {
        if read_byte(&mut byte) {
            if byte == b'\n' {
                break;
            }
            if byte == b'\r' {
                continue;
            }
            buffer[pos] = byte;
            pos += 1;
        } else {
            // No data — stall 1ms and retry. Mirrors C: KeStallExecutionProcessor(1000).
            KeStallExecutionProcessor(1000);
        }
    }
    buffer[pos] = 0;
    pos
}

unsafe fn cstr_eq(buffer: &[u8], len: usize, expected: &[u8]) -> bool {
    // expected includes trailing NUL
    let exp_len = expected.len() - 1;
    if len != exp_len {
        return false;
    }
    buffer[..len] == expected[..exp_len]
}

unsafe fn find_test(name: &[u8], len: usize) -> Option<usize> {
    for (i, entry) in TEST_TABLE.iter().enumerate() {
        if cstr_eq(name, len, entry.name) {
            return Some(i);
        }
    }
    None
}

unsafe fn run_one(idx: usize) {
    let entry = &TEST_TABLE[idx];
    crate::dbg_print!(b"[TEST START] %s\n\0", entry.name.as_ptr());
    let result = (entry.func)();
    if result != 0 {
        crate::dbg_print!(b"[TEST PASS] %s\n\0", entry.name.as_ptr());
    } else {
        crate::dbg_print!(b"[TEST FAIL] %s: test returned FALSE\n\0",
            entry.name.as_ptr(),
        );
    }
}

unsafe fn print_list() {
    crate::dbg_print!(b"[TEST LIST] \0");
    for (i, entry) in TEST_TABLE.iter().enumerate() {
        if i > 0 {
            crate::dbg_print!(b",\0");
        }
        crate::dbg_print!(b"%s\0", entry.name.as_ptr());
    }
    crate::dbg_print!(b"\n\0");
}

/// Main entry point for the test kernel — formerly NtTestKernelDispatchLoop in
/// dispatch.c. Called from Phase1InitializationDiscard when /TESTKERNEL is on
/// the command line. Enters a serial command loop; never launches SMSS.
///
/// **Currently unreachable** — the Rust port of Phase1InitializationDiscard
/// (`ex/init.rs`) doesn't have the `#ifdef REACTOS_TEST_KERNEL` hook that the
/// C version had. Restoring the hook is a small follow-on; see
/// `tests/mod.rs` doc-comment.
#[no_mangle]
pub unsafe extern "system" fn NtTestKernelDispatchLoop() {
    let mut line = [0u8; 256];

    crate::dbg_print!(b"TESTKERNEL READY\n\0");

    loop {
        let len = read_line(&mut line);
        if len == 0 {
            continue;
        }

        if cstr_eq(&line, len, b"QUIT\0") {
            crate::dbg_print!(b"[TEST QUIT]\n\0");
            break;
        }

        if cstr_eq(&line, len, b"LIST\0") {
            print_list();
            continue;
        }

        // RUN <TestName>
        if len >= 4 && &line[..4] == b"RUN " {
            let name_start = 4usize;
            let name_len = len - name_start;
            let name = &line[name_start..len];
            // Append NUL-buffer slice for find_test (it expects a buffer + len).
            // Build a temporary NUL-terminated copy.
            let mut name_buf = [0u8; 128];
            let copy_len = name_len.min(name_buf.len() - 1);
            name_buf[..copy_len].copy_from_slice(&name[..copy_len]);
            // name_buf[copy_len] = 0 (already zeroed)
            match find_test(&name_buf, copy_len) {
                Some(idx) => run_one(idx),
                None => {
                    crate::dbg_print!(b"[TEST FAIL] %s: unknown test\n\0",
                        name_buf.as_ptr(),
                    );
                }
            }
            continue;
        }

        crate::dbg_print!(b"[TEST ERROR] Unknown command: %s\n\0", line.as_ptr());
    }
}
