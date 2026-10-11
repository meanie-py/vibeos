use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn BeginStackBacktrace(data: &mut DumpData) {
    let mut symOptions: DWORD = SymGetOptions();
    symOptions |= SYMOPT_UNDNAME | SYMOPT_AUTO_PUBLICS | SYMOPT_DEFERRED_LOADS;
    SymSetOptions(symOptions);
    SymInitialize(data.ProcessHandle, core::ptr::null_mut(), (TRUE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn EndStackBacktrace(data: &mut DumpData) {
    SymCleanup((*data).ProcessHandle);
}

unsafe extern "C" fn ToChar(data: u8) -> i8 {
    if (data as i32) < 0xa as i32 {
        return (b'0' as i32 + data as i32) as i8;
    } else if (data as i32) <= 0xf as i32 {
        return (b'a' as i32 + data as i32 - 0xa as i32) as i8;
    }
    return b'?' as i8;
}
#[no_mangle]
pub unsafe extern "C" fn PrintStackBacktrace(output: *mut FILE, data: &mut DumpData, thread: &mut ThreadData) {
    let mut MachineType: u32;
    let mut StackFrame: STACKFRAME64 = core::mem::zeroed();
    StackFrame.AddrPC.Offset = 0;

    StackFrame.AddrPC.Mode = AddrModeFlat;
    StackFrame.AddrReturn.Mode = AddrModeFlat;
    StackFrame.AddrFrame.Mode = AddrModeFlat;
    StackFrame.AddrStack.Mode = AddrModeFlat;
    StackFrame.AddrBStore.Mode = AddrModeFlat;

    // One branch per arch, as the C++ `#if defined(_M_IX86) / _M_AMD64 / _M_ARM64` chain.
    #[cfg(target_arch = "x86")]
    {
        MachineType = IMAGE_FILE_MACHINE_I386;
        StackFrame.AddrPC.Offset = thread.Context.Eip as u64;
        StackFrame.AddrStack.Offset = thread.Context.Esp as u64;
        StackFrame.AddrFrame.Offset = thread.Context.Ebp as u64;
    }
    #[cfg(target_arch = "x86_64")]
    {
        MachineType = IMAGE_FILE_MACHINE_AMD64;
        StackFrame.AddrPC.Offset = thread.Context.Rip;
        StackFrame.AddrStack.Offset = thread.Context.Rsp;
        StackFrame.AddrFrame.Offset = thread.Context.Rbp;
    }
    #[cfg(target_arch = "aarch64")]
    {
        MachineType = IMAGE_FILE_MACHINE_ARM64;
        StackFrame.AddrPC.Offset = thread.Context.Pc;
        StackFrame.AddrStack.Offset = thread.Context.Sp;
        StackFrame.AddrFrame.Offset = thread.Context.__bindgen_anon_1.X[29]; // Fp
    }

    let mut buf = [0u8; core::mem::size_of::<SYMBOL_INFO>() + (STACKWALK_MAX_NAMELEN) as usize];
    let sym: *mut SYMBOL_INFO = buf.as_mut_ptr() as *mut SYMBOL_INFO;
    let mut Module: IMAGEHLP_MODULE64 = core::mem::zeroed();
    let mut RecursionDepth: i32 = 0;
    (*sym).SizeOfStruct = core::mem::size_of::<SYMBOL_INFO>() as _;

    xfprintf(output, b"\r\n*----> Stack Back Trace <----*\r\n\r\n\0".as_ptr() as *const i8);
    let mut first: bool = true;
    while StackWalk64(
        MachineType,
        data.ProcessHandle,
        thread.Handle,
        core::ptr::addr_of_mut!(StackFrame),
        core::ptr::addr_of_mut!(thread.Context) as PVOID,
        None,
        Some(SymFunctionTableAccess64),
        Some(SymGetModuleBase64),
        None,
    ) != 0
    {
        if StackFrame.AddrPC.Offset == StackFrame.AddrReturn.Offset {
            let RecursionDepth_old: i32 = RecursionDepth;
            RecursionDepth = RecursionDepth.wrapping_add(1);
            if RecursionDepth_old > STACK_MAX_RECURSION_DEPTH {
                xfprintf(output, b"- Aborting stackwalk -\r\n\0".as_ptr() as *const i8);
                break;
            }
        } else {
            RecursionDepth = 0;
        }

        if first {
            xfprintf(output, b"FramePtr ReturnAd Param#1  Param#2  Param#3  Param#4  Function Name\r\n\0".as_ptr() as *const i8);
            first = false;
        }

        Module.SizeOfStruct = core::mem::size_of::<IMAGEHLP_MODULE64>() as _;
        let ModBase: DWORD64 = SymGetModuleBase64(data.ProcessHandle, StackFrame.AddrPC.Offset);
        if ModBase == 0 || SymGetModuleInfo64(data.ProcessHandle, ModBase, core::ptr::addr_of_mut!(Module)) == 0 {
            strcpy(core::ptr::addr_of_mut!(Module.ModuleName) as *mut i8, b"<nomod>\0".as_ptr() as *const i8);
        }

        memset(sym as *mut core::ffi::c_void, 0, core::mem::size_of::<SYMBOL_INFO>() + (STACKWALK_MAX_NAMELEN) as usize);
        (*sym).SizeOfStruct = core::mem::size_of::<SYMBOL_INFO>() as _;
        (*sym).MaxNameLen = STACKWALK_MAX_NAMELEN as _;
        let mut displacement: DWORD64 = 0;

        if StackFrame.AddrPC.Offset == 0
            || SymFromAddr(data.ProcessHandle, StackFrame.AddrPC.Offset, core::ptr::addr_of_mut!(displacement), sym) == 0
        {
            strcpy(core::ptr::addr_of_mut!((*sym).Name) as *mut i8, b"<nosymbols>\0".as_ptr() as *const i8);
        }

        xfprintf(
            output,
            b"%p %p %p %p %p %p %s!%s +0x%I64x\r\n\0".as_ptr() as *const i8,
            StackFrame.AddrFrame.Offset as usize,
            StackFrame.AddrPC.Offset as usize,
            StackFrame.Params[0 as usize] as usize,
            StackFrame.Params[1 as usize] as usize,
            StackFrame.Params[2 as usize] as usize,
            StackFrame.Params[3 as usize] as usize,
            core::ptr::addr_of!(Module.ModuleName) as *const i8,
            core::ptr::addr_of!((*sym).Name) as *const i8,
            displacement,
        );
    }

    let mut stackData = [0u8; 0x10 * 10];
    let mut sizeRead: usize = 0;
    #[cfg(target_arch = "x86")]
    let stackPointer: usize = thread.Context.Esp as usize;
    #[cfg(target_arch = "x86_64")]
    let stackPointer: usize = thread.Context.Rsp as usize;
    #[cfg(target_arch = "aarch64")]
    let stackPointer: usize = thread.Context.Sp as usize;
    if ReadProcessMemory(
        data.ProcessHandle,
        stackPointer as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(stackData) as *mut core::ffi::c_void,
        0x10 * 10,
        (core::ptr::addr_of_mut!(sizeRead)).cast(),
    ) == 0
    {
        return;
    }

    xfprintf(output, b"\r\n*----> Raw Stack Dump <----*\r\n\r\n\0".as_ptr() as *const i8);
    let mut n: usize = 0;
    while n < 0x10 * 10 {
        let mut HexData1: [u8; 24] = *b"?? ?? ?? ?? ?? ?? ?? ??\0";
        let mut HexData2: [u8; 24] = *b"?? ?? ?? ?? ?? ?? ?? ??\0";
        let mut AsciiData1: [u8; 9] = *b"????????\0";
        let mut AsciiData2: [u8; 9] = *b"????????\0";

        let mut j: usize = 0;
        while j < 8 {
            let mut idx: usize = j + n;
            if idx < sizeRead {
                HexData1[j * (3 as usize)] = ToChar(stackData[idx] >> 4) as u8;
                HexData1[j * (3 as usize) + (1 as usize)] = ToChar(stackData[idx] & 0xf) as u8;
                AsciiData1[j] = if isprint(stackData[idx] as i32) != 0 { stackData[idx] } else { b'.' };
            }
            idx += 8 as usize;
            if idx < sizeRead {
                HexData2[j * (3 as usize)] = ToChar(stackData[idx] >> 4) as u8;
                HexData2[j * (3 as usize) + (1 as usize)] = ToChar(stackData[idx] & 0xf) as u8;
                AsciiData2[j] = if isprint(stackData[idx] as i32) != 0 { stackData[idx] } else { b'.' };
            }
            j += 1;
        }

        xfprintf(
            output,
            b"%p %s - %s  %s%s\r\n\0".as_ptr() as *const i8,
            stackPointer + n,
            core::ptr::addr_of!(HexData1) as *const i8,
            core::ptr::addr_of!(HexData2) as *const i8,
            core::ptr::addr_of!(AsciiData1) as *const i8,
            core::ptr::addr_of!(AsciiData2) as *const i8,
        );
        n += 0x10 as usize;
    }
}
